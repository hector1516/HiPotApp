"""SQLAlchemy models — HiPotApp (docs 04, 08, 09 + 07 admin)."""
from datetime import datetime
from sqlalchemy import (
    Column, Integer, BigInteger, String, Unicode, UnicodeText,
    Float, Boolean, DateTime, ForeignKey, Index,
)
from sqlalchemy.orm import declarative_base, relationship

Base = declarative_base()


class Recipe(Base):
    __tablename__ = "recipes"
    id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(Unicode(100), nullable=False)
    model = Column(Unicode(50), unique=True, nullable=False)
    type = Column(Unicode(50), default="current_consumption")
    product = Column(Unicode(100), nullable=True)
    voltage_levels = Column(UnicodeText, nullable=True)  # JSON list
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.now)
    updated_at = Column(DateTime, default=datetime.now, onupdate=datetime.now)
    modified_by = Column(Unicode(50), nullable=True)  # batch del admin que guardó
    components = relationship("RecipeComponent", back_populates="recipe", cascade="all, delete-orphan")
    totals = relationship("RecipeTotal", back_populates="recipe", cascade="all, delete-orphan")


class RecipeComponent(Base):
    __tablename__ = "recipe_components"
    id = Column(Integer, primary_key=True, autoincrement=True)
    recipe_id = Column(Integer, ForeignKey("recipes.id"), nullable=False)
    name = Column(Unicode(50), nullable=False)       # FAN, DOOR, FRAME...
    voltage = Column(Unicode(20), nullable=False)    # 120VAC...
    enabled = Column(Boolean, default=True)
    min_value = Column(Float, default=0)
    max_value = Column(Float, default=0)
    unit = Column(Unicode(10), default="A")
    recipe = relationship("Recipe", back_populates="components")


class RecipeTotal(Base):
    __tablename__ = "recipe_totals"
    id = Column(Integer, primary_key=True, autoincrement=True)
    recipe_id = Column(Integer, ForeignKey("recipes.id"), nullable=False)
    voltage = Column(Unicode(20), nullable=False)
    min_total = Column(Float, nullable=False)
    max_total = Column(Float, nullable=False)
    recipe = relationship("Recipe", back_populates="totals")


class Operator(Base):
    __tablename__ = "operators"
    id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(Unicode(100), nullable=False)
    employee_id = Column(Unicode(20), unique=True, nullable=False)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.now)


class Batch(Base):
    __tablename__ = "batches"
    id = Column(Integer, primary_key=True, autoincrement=True)
    batch_number = Column(Unicode(50), unique=True, nullable=False)
    operator_id = Column(Integer, ForeignKey("operators.id"), nullable=True)
    scanned_at = Column(DateTime, default=datetime.now)


class Execution(Base):
    __tablename__ = "executions"
    id = Column(Integer, primary_key=True, autoincrement=True)
    recipe_id = Column(Integer, ForeignKey("recipes.id"), nullable=True)
    operator_id = Column(Integer, ForeignKey("operators.id"), nullable=True)
    batch_id = Column(Integer, ForeignKey("batches.id"), nullable=True)
    serial_number = Column(Unicode(100), nullable=True)
    plc_start_time = Column(DateTime, nullable=True)
    plc_end_time = Column(DateTime, nullable=True)
    plc_duration = Column(Float, nullable=True)
    plc_pass_fail = Column(Boolean, nullable=True)
    hipot_start_time = Column(DateTime, nullable=True)
    hipot_end_time = Column(DateTime, nullable=True)
    hipot_duration = Column(Float, nullable=True)
    hipot_pass_fail = Column(Boolean, nullable=True)
    final_status = Column(Unicode(10), nullable=True)  # GOOD / BAD
    total_duration = Column(Float, nullable=True)
    synced_to_remote = Column(Boolean, default=False)
    synced_at = Column(DateTime, nullable=True)
    created_at = Column(DateTime, default=datetime.now)


class PlcReading(Base):
    __tablename__ = "plc_readings"
    id = Column(Integer, primary_key=True, autoincrement=True)
    execution_id = Column(Integer, ForeignKey("executions.id"), nullable=True)
    recipe_id = Column(Integer, ForeignKey("recipes.id"), nullable=True)
    component_name = Column(Unicode(50), nullable=True)
    voltage = Column(Unicode(20), nullable=True)
    tag_name = Column(Unicode(50), nullable=True)
    value = Column(Float, nullable=True)
    min_limit = Column(Float, nullable=True)
    max_limit = Column(Float, nullable=True)
    is_pass = Column(Boolean, nullable=True)
    timestamp = Column(DateTime, default=datetime.now)


class HipotResult(Base):
    __tablename__ = "hipot_results"
    id = Column(Integer, primary_key=True, autoincrement=True)
    execution_id = Column(Integer, ForeignKey("executions.id"), nullable=True)
    recipe_id = Column(Integer, ForeignKey("recipes.id"), nullable=True)
    voltage = Column(Float, nullable=True)
    current = Column(Float, nullable=True)
    resistance = Column(Float, nullable=True)
    insulation_resistance = Column(Float, nullable=True)
    test_duration = Column(Float, nullable=True)
    is_pass = Column(Boolean, nullable=True)
    timestamp = Column(DateTime, default=datetime.now)


class BarcodeLog(Base):
    __tablename__ = "barcode_logs"
    id = Column(Integer, primary_key=True, autoincrement=True)
    code = Column(Unicode(100), nullable=False)
    barcode_type = Column(Unicode(20), nullable=True)  # model, serial, batch
    scan_order = Column(Integer, nullable=True)
    execution_id = Column(Integer, ForeignKey("executions.id"), nullable=True)
    scanned_at = Column(DateTime, default=datetime.now)


class Configuration(Base):
    __tablename__ = "configurations"
    id = Column(Integer, primary_key=True, autoincrement=True)
    config_key = Column(Unicode(50), unique=True, nullable=False)
    config_value = Column(Unicode(500), nullable=False)
    updated_at = Column(DateTime, default=datetime.now, onupdate=datetime.now)


class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True, autoincrement=True)
    username = Column(Unicode(50), unique=True, nullable=False)
    password_hash = Column(Unicode(255), nullable=False)
    role = Column(Unicode(20), default="operator")
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.now)


class SyncLog(Base):
    __tablename__ = "sync_log"
    id = Column(Integer, primary_key=True, autoincrement=True)
    direction = Column(Unicode(10), nullable=True)
    table_name = Column(Unicode(50), nullable=True)
    records_synced = Column(Integer, default=0)
    status = Column(Unicode(20), nullable=True)
    error_message = Column(Unicode(500), nullable=True)
    started_at = Column(DateTime, default=datetime.now)
    completed_at = Column(DateTime, nullable=True)


class AppLog(Base):
    __tablename__ = "app_logs"
    id = Column(Integer, primary_key=True, autoincrement=True)
    timestamp = Column(DateTime, default=datetime.now)
    level = Column(Unicode(10), nullable=False)       # INFO, WARN, ERROR, DEBUG
    category = Column(Unicode(50), nullable=False)    # UI, PLC, HIPOT, DB, SYNC, BARCODE, SYSTEM
    action = Column(Unicode(100), nullable=False)
    detail = Column(UnicodeText, nullable=True)
    execution_id = Column(Integer, nullable=True)
    user_id = Column(Integer, nullable=True)
    value_sent = Column(Unicode(200), nullable=True)
    value_received = Column(Unicode(200), nullable=True)
    duration_ms = Column(Integer, nullable=True)
    success = Column(Boolean, default=True)
    error_message = Column(Unicode(500), nullable=True)


class UiStateLog(Base):
    __tablename__ = "ui_state_log"
    id = Column(Integer, primary_key=True, autoincrement=True)
    timestamp = Column(DateTime, default=datetime.now)
    execution_id = Column(Integer, nullable=True)
    current_step = Column(Integer, nullable=True)
    current_phase = Column(Unicode(20), nullable=True)
    background_color = Column(Unicode(20), nullable=True)
    message_displayed = Column(Unicode(200), nullable=True)


class LabelTemplate(Base):
    __tablename__ = "label_templates"
    id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(Unicode(100), nullable=False)
    width_mm = Column(Integer, default=100)
    height_mm = Column(Integer, default=60)
    canvas_data = Column(UnicodeText, nullable=True)  # JSON del editor visual
    is_default = Column(Boolean, default=False)
    created_at = Column(DateTime, default=datetime.now)
    updated_at = Column(DateTime, default=datetime.now, onupdate=datetime.now)


class LabelPrint(Base):
    __tablename__ = "label_prints"
    id = Column(Integer, primary_key=True, autoincrement=True)
    execution_id = Column(Integer, ForeignKey("executions.id"), nullable=True)
    template_id = Column(Integer, ForeignKey("label_templates.id"), nullable=True)
    printed_by = Column(Unicode(50), default="auto")
    printed_at = Column(DateTime, default=datetime.now)
    printer_model = Column(Unicode(50), default="Zebra ZT231")
    connection_type = Column(Unicode(10), default="usb")
    zpl_data = Column(UnicodeText, nullable=True)
    status = Column(Unicode(20), nullable=True)
    error_message = Column(Unicode(200), nullable=True)
    copies = Column(Integer, default=1)


class PlcTag(Base):
    __tablename__ = "plc_tags"
    id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(Unicode(50), nullable=False)
    db_number = Column(Integer, default=1)
    offset = Column(Integer, default=0)
    data_type = Column(Unicode(10), default="REAL")  # REAL, BOOL, WORD
    bit = Column(Integer, default=0)
    app_variable = Column(Unicode(50), nullable=True, default="CUSTOM")
    description = Column(Unicode(200), nullable=True)
    unit = Column(Unicode(10), nullable=True)


class ImportExportLog(Base):
    __tablename__ = "import_export_logs"
    id = Column(Integer, primary_key=True, autoincrement=True)
    operation_type = Column(Unicode(10), nullable=False)  # import / export
    user_id = Column(Integer, ForeignKey("users.id"), nullable=True)
    file_name = Column(Unicode(200), nullable=True)
    file_size_bytes = Column(BigInteger, nullable=True)
    data_summary = Column(UnicodeText, nullable=True)
    mode = Column(Unicode(20), nullable=True)
    conflicts_found = Column(Integer, default=0)
    records_imported = Column(Integer, default=0)
    status = Column(Unicode(20), default="success")
    error_message = Column(Unicode(500), nullable=True)
    performed_at = Column(DateTime, default=datetime.now)


Index("idx_app_logs_timestamp", AppLog.timestamp)
Index("idx_app_logs_category", AppLog.category)
Index("idx_app_logs_level", AppLog.level)
Index("idx_app_logs_execution", AppLog.execution_id)

DEFAULT_CONFIGS = [
    ("plc_ip", "10.188.141.10"),
    ("plc_rack", "0"),
    ("plc_slot", "1"),
    ("plc_simulate", "False"),
    ("hipot_port", "COM3"),
    ("hipot_baud", "9600"),
    ("hipot_parity", "None"),
    ("hipot_databits", "8"),
    ("hipot_stopbits", "1"),
    ("db_local_server", ".\\SQLEXPRESS"),
    ("db_local_name", "HiPotDB"),
    ("db_remote_server", "192.168.1.200"),
    ("db_remote_name", "HiPotDB_Central"),
    ("sync_interval_minutes", "5"),
    ("sync_mode", "local_to_remote"),
    ("printer_model", "Zebra ZT231"),
    ("label_width_mm", "100"),
    ("label_height_mm", "60"),
    ("auto_print", "1"),
    ("print_only_good", "1"),
    ("result_hold_seconds", "10"),
]
