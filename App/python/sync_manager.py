"""Sync local -> remoto (doc 04 §5). Offline-first."""
import asyncio
from datetime import datetime


class SyncManager:
    SYNC_TABLES = ["executions", "plc_readings", "hipot_results", "barcode_logs",
                   "recipes", "operators", "batches", "users", "label_prints"]

    def __init__(self, local_session_factory, remote_session_factory=None,
                 interval_minutes: int = 5):
        self.local_sf = local_session_factory
        self.remote_sf = remote_session_factory
        self.interval = interval_minutes * 60
        self.running = False

    async def start_loop(self):
        self.running = True
        while self.running:
            try:
                await asyncio.to_thread(self.sync_pending)
            except Exception:
                pass
            await asyncio.sleep(self.interval)

    def sync_pending(self) -> int:
        """Copia executions pendientes al remoto. Retorna nº sincronizados."""
        if not self.remote_sf:
            return 0
        from .models import Execution, SyncLog
        ls, rs = self.local_sf(), self.remote_sf()
        n = 0
        start = datetime.now()
        try:
            pend = ls.query(Execution).filter(Execution.synced_to_remote == False).all()  # noqa: E712
            for rec in pend:
                try:
                    data = {c.name: getattr(rec, c.name)
                            for c in Execution.__table__.columns if c.name != "id"}
                    rs.execute(Execution.__table__.insert().values(**data))
                    rs.commit()
                    rec.synced_to_remote = True
                    rec.synced_at = datetime.now()
                    ls.commit()
                    n += 1
                except Exception as e:
                    rs.rollback()
                    ls.add(SyncLog(direction="local_to_remote", table_name="executions",
                                   records_synced=0, status="error",
                                   error_message=str(e)[:500], completed_at=datetime.now()))
                    ls.commit()
            ls.add(SyncLog(direction="local_to_remote", table_name="executions",
                            records_synced=n, status="success",
                            completed_at=datetime.now()))
            ls.commit()
        finally:
            ls.close(); rs.close()
        return n
