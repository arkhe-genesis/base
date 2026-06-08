import pytest
from arkhe_cathedral_orchestrator_v5.cathedral_orchestrator_v5 import CathedralOrchestratorV5

def test_cathedral_orchestrator():
    orch = CathedralOrchestratorV5()
    # basic init
    assert orch.cycle_count == 0

def test_orchestrator_components():
    orch = CathedralOrchestratorV5()
    # verify initializations are unpopulated initially
    assert orch.vt is None
    assert orch.stethoscope is None
    assert orch.kleros is None
