#!/usr/bin/env python3
"""
Helper script to send transactions to deployed contracts.
Used by deploy.sh to create test transactions.
"""

import sys
import json
import requests
from eth_utils import to_checksum_address, function_signature_to_4byte_selector

def send_transaction(rpc_url, from_addr, to_addr, function_sig, params=""):
    """Send a transaction and return the hash."""
    
    # Get function selector
    if function_sig:
        selector = function_signature_to_4byte_selector(function_sig).hex()
    else:
        selector = ""
    
    # Construct data
    data = "0x" + selector + params
    
    # Prepare transaction
    tx = {
        "from": from_addr,
        "to": to_checksum_address(to_addr),
        "data": data,
        "gas": "0x100000",  # 1M gas
        "gasPrice": "0x3b9aca00"  # 1 gwei
    }
    
    # Send RPC request
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_sendTransaction",
        "params": [tx],
        "id": 1
    }
    
    try:
        response = requests.post(rpc_url, json=payload)
        result = response.json()
        
        if "result" in result:
            return result["result"]
        else:
            print(f"Error: {result.get('error', 'Unknown error')}", file=sys.stderr)
            return None
    except Exception as e:
        print(f"Error sending transaction: {e}", file=sys.stderr)
        return None

def wait_for_receipt(rpc_url, tx_hash, timeout=30):
    """Wait for transaction receipt."""
    import time
    
    for _ in range(timeout):
        payload = {
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [tx_hash],
            "id": 1
        }
        
        try:
            response = requests.post(rpc_url, json=payload)
            result = response.json()
            
            if result.get("result"):
                return result["result"]
        except:
            pass
        
        time.sleep(1)
    
    return None

if __name__ == "__main__":
    if len(sys.argv) < 5:
        print("Usage: send_tx.py <rpc_url> <from_addr> <to_addr> <function_sig> [params]")
        sys.exit(1)
    
    rpc_url = sys.argv[1]
    from_addr = sys.argv[2]
    to_addr = sys.argv[3]
    function_sig = sys.argv[4]
    params = sys.argv[5] if len(sys.argv) > 5 else ""
    
    tx_hash = send_transaction(rpc_url, from_addr, to_addr, function_sig, params)
    
    if tx_hash:
        print(tx_hash)
        
        # Wait for receipt
        receipt = wait_for_receipt(rpc_url, tx_hash)
        if receipt and receipt.get("status") == "0x1":
            sys.exit(0)
        else:
            sys.exit(1)
    else:
        sys.exit(1)