"""DB dual: SQL Express local + SQL Server remoto, con fallback SQLite para dev."""
import os
import sys
from sqlalchemy import create_engine, text
from sqlalchemy.orm import sessionmaker
from .models import Base, DEFAULT_CONFIGS, Configuration


def _writable_base() -> str:
    """Retorna un directorio persistente en AppData para el usuario en Windows."""
    appdata = os.getenv("APPDATA") or os.path.expanduser("~")
    base_dir = os.path.join(appdata, "HiPotApp")
    os.makedirs(base_dir, exist_ok=True)
    return base_dir


def _build_mssql_url(server: str, db: str, user: str = "", password: str = "") -> str:
    driver = os.getenv("MSSQL_DRIVER", "ODBC Driver 17 for SQL Server").replace(" ", "+")
    if user:
        return f"mssql+pyodbc://{user}:{password}@{server}/{db}?driver={driver}"
    return f"mssql+pyodbc://@{server}/{db}?driver={driver}&trusted_connection=yes"


def get_database_url(which: str = "local") -> str:
    if which == "remote":
        if os.getenv("REMOTE_DATABASE_URL"):
            return os.getenv("REMOTE_DATABASE_URL")
        server = os.getenv("DB_REMOTE_SERVER", "192.168.1.200")
        name = os.getenv("DB_REMOTE_NAME", "HiPotDB_Central")
        user = os.getenv("DB_REMOTE_USER", "")
        pwd = os.getenv("DB_REMOTE_PASSWORD", "")
        try_url = _build_mssql_url(server, name, user, pwd)
        return os.getenv("REMOTE_FALLBACK_URL", try_url)
    # local
    if os.getenv("DATABASE_URL"):
        return os.getenv("DATABASE_URL")
    if os.getenv("USE_SQLITE", "1") == "1":
        return f"sqlite:///{os.path.join(_writable_base(), 'hipot_local.db')}"
    server = os.getenv("DB_LOCAL_SERVER", ".\\SQLEXPRESS")
    name = os.getenv("DB_LOCAL_NAME", "HiPotDB")
    return _build_mssql_url(server, name)


class DatabaseManager:
    def __init__(self, url: str | None = None):
        self.url = url or get_database_url("local")
        connect_args = {"check_same_thread": False} if self.url.startswith("sqlite") else {}
        self.engine = create_engine(self.url, pool_pre_ping=True, connect_args=connect_args)
        self.Session = sessionmaker(bind=self.engine)
        self.session = None

    def connect(self) -> bool:
        try:
            self.session = self.Session()
            self.session.execute(text("SELECT 1"))
            return True
        except Exception:
            return False

    def create_tables(self) -> dict:
        Base.metadata.create_all(self.engine)
        # Migraciones suaves para esquemas existentes
        try:
            with self.engine.connect() as c:
                if self.url.startswith("sqlite"):
                    res = c.execute(text("PRAGMA table_info(recipes)"))
                    cols = [r[1] for r in res]
                    if "modified_by" not in cols:
                        c.execute(text("ALTER TABLE recipes ADD COLUMN modified_by NVARCHAR(50)"))
                        c.commit()
                    res_tags = c.execute(text("PRAGMA table_info(plc_tags)"))
                    cols_tags = [r[1] for r in res_tags]
                    if "app_variable" not in cols_tags:
                        c.execute(text("ALTER TABLE plc_tags ADD COLUMN app_variable NVARCHAR(50)"))
                        c.commit()
                else:
                    c.execute(text("IF COL_LENGTH('recipes','modified_by') IS NULL ALTER TABLE recipes ADD modified_by NVARCHAR(50)"))
                    c.execute(text("IF COL_LENGTH('plc_tags','app_variable') IS NULL ALTER TABLE plc_tags ADD app_variable NVARCHAR(50)"))
                    c.commit()
        except Exception:
            pass
        created = len(Base.metadata.tables)
        self.seed_defaults()
        return {"tables": created}

    def seed_defaults(self):
        s = self.Session()
        try:
            for k, v in DEFAULT_CONFIGS:
                if not s.query(Configuration).filter_by(config_key=k).first():
                    s.add(Configuration(config_key=k, config_value=v))
            s.commit()
        except Exception:
            s.rollback()
        finally:
            s.close()

    def test_connection(self) -> bool:
        try:
            with self.engine.connect() as c:
                c.execute(text("SELECT 1"))
            return True
        except Exception:
            return False


local_db = DatabaseManager(get_database_url("local"))


def get_remote_db() -> DatabaseManager:
    return DatabaseManager(get_database_url("remote"))
