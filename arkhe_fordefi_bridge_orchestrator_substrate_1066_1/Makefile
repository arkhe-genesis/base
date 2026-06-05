.PHONY: install test deploy clean

install:
	pip install -e .
	pip install web3 pyyaml

test:
	python -m pytest tests/ -v
	bash scripts/test_integration.sh

deploy:
	echo "[DEPLOY] Compilando FordefiBridgeAnchor.sol..."
	solc --bin --abi contracts/FordefiBridgeAnchor.sol -o build/
	echo "[DEPLOY] Deploy na RBB Chain (12120014)..."
	# Em produção: web3 deploy script

clean:
	rm -rf build/ dist/ *.egg-info
	rm -rf ~/.cathedral/fordefi_*
