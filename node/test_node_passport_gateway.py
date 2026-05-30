import pytest
from passport_gateway import PassportGateway, HumanityProof

@pytest.mark.asyncio
async def testverify_orcid_link():
    gateway = PassportGateway()

    # Simulate an address with an ORCID
    result_alice = await gateway.verify_orcid_link("0xAlice123")
    assert result_alice is True

    # Simulate an address without an ORCID
    result_bob = await gateway.verify_orcid_link("0xBob123")
    assert result_bob is False
