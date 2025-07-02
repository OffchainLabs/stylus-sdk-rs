{
    "hostio": function(info) {
        info.args = toHex(info.args);
        info.outs = toHex(info.outs);
        if (this.nests.includes(info.name)) {
            Object.assign(info, this.open.pop());
            info.name = info.name.substring(4) // remove evm_
        }
        this.open.push(info);
    },
    "enter": function(frame) {
        let inner = [];
        let name = "";
        switch (frame.getType()) {
        case "CALL":
                name = "evm_call_contract";
                break;
        case "DELEGATECALL":
                name = "evm_delegate_call_contract";
                break;
        case "STATICCALL":
                name = "evm_static_call_contract";
                break;
        case "CREATE":
                name = "evm_create1";
                break;
        case "CREATE2":
                name = "evm_create2";
                break;
        case "SELFDESTRUCT":
                name = "evm_self_destruct";
                break;
        }
        this.open.push({
            address: toHex(frame.getTo()),
            steps: inner,
            name: name,
        });
        this.stack.push(this.open); // save where we were
        this.open = inner;
    },
    "exit": function(result) {
        this.open = this.stack.pop();
    },
    "result": function() { return this.open; },
    "fault":  function() { return this.open; },
    stack: [],
    open: [],
    nests: ["call_contract", "delegate_call_contract", "static_call_contract"]
}
