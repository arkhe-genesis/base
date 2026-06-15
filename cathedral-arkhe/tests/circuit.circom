pragma circom 2.0.0;

template Multiplier2() {
    signal input a;
    signal input b;
    signal output c;

    // A bug to test zkHydra:
    // Missing constraint c <== a * b;
    c <-- a * b; // only assignment, no constraint
}

component main = Multiplier2();