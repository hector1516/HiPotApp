"""
FastAPI Backend for HiPod Application.
Handles PLC communication (Siemens S7-1200 via snap7),
Hypot 3865 serial communication, Database operations (PostgreSQL),
Recipe management, and Barcode scanning integration.
"""

from fastapi import FastAPI, HTTPException, Depends, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
from typing import Optional, Dict, List, Any
import asyncio
import json
import time
import os

# Initialize FastAPI app
app = FastAPI(
    title="HiPod API",
    description="Tauri + Python backend for Siemens PLC + Hypot 3865 communication",
    version="1.0.0"
)

# CORS configuration for Tauri
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# ============================================================
# DATA MODELS (SQLAlchemy models will be separate)
# ============================================================

class Recipe(BaseModel):
    id: Optional[int] = None
    name: str
    model: str
    steps: Dict[str, Any] = Field(default_factory=dict)
    is_active: bool = True
    created_at: Optional[str] = None

class Operator(BaseModel):
    id: Optional[int] = None
    name: str
    employee_id: str
    is_active: bool = True

class Batch(BaseModel):
    id: Optional[int] = None
    batch_number: str
    operator_id: Optional[int] = None
    scanned_at: Optional[str] = None

class BarcodeScan(BaseModel):
    code: str
    barcode_type: str = "model"  # model, serial, batch
    scan_order: int = 1  # 1=model, 2=serial, 3=batch

class PlcResult(BaseModel):
    recipe_id: int
    tag_name: str
    value: float
    timestamp: str
    is_pass: bool

class HypotResult(BaseModel):
    recipe_id: int
    voltage: float
    current: float
    resistance: float
    insulation_resistance: float
    test_duration: float
    timestamp: str
    is_pass: bool

class ExecutionSummary(BaseModel):
    execution_id: int
    recipe_id: int
    operator_id: int
    batch_id: Optional[int] = None
    plc_start: str
    plc_end: str
    hipot_start: str
    hipot_end: str
    final_status: str  # "GOOD" or "BAD"
    total_duration: float

# ============================================================
# IN-MEMORY STORAGE (will be replaced by PostgreSQL)
# ============================================================

recipes_db: Dict[int, Recipe] = {}
operators_db: Dict[int, Operator] = {}
barcode_logs: List[BarcodeScan] = []
execution_logs: List[ExecutionSummary] = []

# ============================================================
# API Routes
# ============================================================

@app.get("/")
async def root():
    return {"message": "HiPod API is running", "version": "1.0.0"}

@app.get("/health")
async def health_check():
    return {"status": "healthy", "services": {"plc": "disconnected", "hypot": "disconnected", "database": "disconnected"}}

# ---- Recipe Routes ----

@app.get("/recipes", response_model=List[Recipe])
async def get_recipes():
    """Get all recipes from database"""
    return [r for r in recipes_db.values() if r.is_active]

@app.post("/recipes")
async def create_recipe(recipe: Recipe):
    """Create new recipe"""
    recipe.id = max(recipes_db.keys()) + 1 if recipes_db else 1
    recipes_db[recipe.id] = recipe
    return {"status": "created", "recipe_id": recipe.id}

@app.get("/recipes/model/{model}")
async def get_recipe_by_model(model: str):
    """Find recipe by model barcode"""
    for r in recipes_db.values():
        if r.model == model and r.is_active:
            return r
    raise HTTPException(status_code=404, detail=f"Recipe not found for model: {model}")

# ---- Operator Routes ----

@app.get("/operators")
async def get_operators():
    """Get all operators"""
    return [o for o in operators_db.values() if o.is_active]

@app.post("/operators")
async def create_operator(operator: Operator):
    """Create new operator"""
    operator.id = max(operators_db.keys()) + 1 if operators_db else 1
    operators_db[operator.id] = operator
    return {"status": "created", "operator_id": operator.id}

# ---- Barcode Routes ----

@app.post("/barcode/scan")
async def scan_barcode(scan: BarcodeScan):
    """Log barcode scan"""
    scan.id = len(barcode_logs) + 1
    barcode_logs.append(scan)
    return {"status": "scanned", "code": scan.code, "order": scan.scan_order}

@app.get("/barcode/logs")
async def get_barcode_logs():
    """Get all barcode logs"""
    return barcode_logs

# ---- Execution Routes ----

@app.post("/executions/start")
async def start_execution(
    recipe_id: int,
    operator_id: int,
    batch_id: Optional[int] = None
):
    """Start a new execution"""
    execution_id = len(execution_logs) + 1
    execution_logs.append(ExecutionSummary(
        execution_id=execution_id,
        recipe_id=recipe_id,
        operator_id=operator_id,
        batch_id=batch_id,
        plc_start=time.strftime("%Y-%m-%d %H:%M:%S"),
        plc_end="",
        hipot_start="",
        hipot_end="",
        final_status="PENDING",
        total_duration=0.0
    ))
    return {"execution_id": execution_id, "status": "started"}

@app.post("/executions/complete")
async def complete_execution(
    execution_id: int,
    plc_pass: bool,
    hipot_pass: bool
):
    """Complete execution and determine final status"""
    if execution_id > len(execution_logs):
        raise HTTPException(status_code=404, detail="Execution not found")
    
    exec_log = execution_logs[execution_id - 1]
    
    # Determine final status
    if plc_pass and hipot_pass:
        final_status = "GOOD"
    else:
        final_status = "BAD"
    
    # Calculate durations
    from datetime import datetime
    plc_start = datetime.strptime(exec_log.plc_start, "%Y-%m-%d %H:%M:%S")
    plc_end = datetime.now()
    plc_duration = (plc_end - plc_start).total_seconds()
    
    # Update execution log
    exec_log.plc_end = time.strftime("%Y-%m-%d %H:%M:%S")
    exec_log.hipot_start = exec_log.plc_end  # Simplified - would be after PLC
    exec_log.hipot_end = time.strftime("%Y-%m-%d %H:%M:%S")
    exec_log.final_status = final_status
    exec_log.total_duration = plc_duration + 30  # Approximate, would calculate properly
    
    return {
        "status": "completed", 
        "execution_id": execution_id,
        "final_status": final_status,
        "plc_duration": plc_duration,
        "total_duration": exec_log.total_duration
    }

# ---- Health/Status Routes ----

@app.get("/status/plc")
async def plc_status():
    """PLC connection status"""
    return {"connected": False, "details": "Will connect via snap7 when initialized"}

@app.get("/status/hypot")
async def hipot_status():
    """Hypot connection status"""
    return {"connected": False, "details": "Will connect via pyserial when initialized"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)