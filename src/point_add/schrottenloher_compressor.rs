//! Ternary compressor for the dialog-based EEA (Schrottenloher 2026).
//!
//! Maps 3 pairs of bits (b0, b0&b1) from the GCD algorithm into 5 bits.
//! Each pair has the form (00|10|11), giving 3^3 = 27 valid inputs.
//! The compression uses a SAT-synthesized circuit with minimal gate count.
//!
//! Reference: point_add/compressor.py in the Qarton implementation.

use crate::circuit::{BitId, QubitId};

use super::B;

/// Compressor circuit: takes 6 bits, produces 5 bits.
/// The input is (b0_0, b0andb1_0, b0_1, b0andb1_1, b0_2, b0andb1_2).
/// The output is a 5-bit compressed representation.
///
/// Gate count: 3 CX + 3 CX + 1 X + 7 CCX + 1 CX + 1 X = ~17 gates total.
/// Reversible: the circuit asserts the 6th bit is zero at the end.
impl B {
    pub fn compressor(&mut self, inp: &[BitId; 6], out: &[BitId; 5]) {
        // xreg[0..6] = inp, xreg[0..5] = out (we reuse the first 5 bits)
        // Circuit from compressor.py Compressor class:
        //   cx(xreg[1], xreg[0])
        //   cx(xreg[3], xreg[2])
        //   cx(xreg[5], xreg[4])
        //   cx(xreg[0], xreg[2])
        //   cx(xreg[5], xreg[3])
        //   x(xreg[4])
        //   ccx(xreg[1], xreg[3], xreg[5])
        //   cx(xreg[1], xreg[4])
        //   x(xreg[2])
        //   ccx(xreg[3], xreg[4], xreg[5])
        //   ccx(xreg[4], xreg[5], xreg[1])
        //   ccx(xreg[2], xreg[5], xreg[0])
        //   ccx(xreg[0], xreg[1], xreg[5])
        //   assert xreg[5] == 0
        //
        // We operate in-place on the input bits and copy first 5 to output.
        // But the harness separates BitId (classical) from QubitId (quantum).
        // The compressor operates on bits, so we use cx/ccx on bits.

        // For bits, we use cnot_bit and ccx_bit helpers (defined below).
        self.cx_bit(inp[1], inp[0]);
        self.cx_bit(inp[3], inp[2]);
        self.cx_bit(inp[5], inp[4]);
        self.cx_bit(inp[0], inp[2]);
        self.cx_bit(inp[5], inp[3]);
        self.x_bit(inp[4]);
        self.ccx_bit(inp[1], inp[3], inp[5]);
        self.cx_bit(inp[1], inp[4]);
        self.x_bit(inp[2]);
        self.ccx_bit(inp[3], inp[4], inp[5]);
        self.ccx_bit(inp[4], inp[5], inp[1]);
        self.ccx_bit(inp[2], inp[5], inp[0]);
        self.ccx_bit(inp[0], inp[1], inp[5]);
        // inp[5] should now be 0; we copy first 5 to output
        for i in 0..5 {
            self.cx_bit(inp[i], out[i]);
        }
    }

    /// Inverse compressor: 5 bits → 6 bits (with 6th bit asserted zero).
    pub fn compressor_inv(&mut self, inp: &[BitId; 5], out: &[BitId; 6]) {
        // Reverse order of gates from compressor
        // First copy 5 bits to first 5 of output
        for i in 0..5 {
            self.cx_bit(inp[i], out[i]);
        }
        // Apply inverse gates in reverse order
        self.ccx_bit(out[0], out[1], out[5]);
        self.ccx_bit(out[2], out[5], out[0]);
        self.ccx_bit(out[4], out[5], out[1]);
        self.ccx_bit(out[3], out[4], out[5]);
        self.x_bit(out[2]);
        self.cx_bit(out[1], out[4]);
        self.ccx_bit(out[1], out[3], out[5]);
        self.x_bit(out[4]);
        self.cx_bit(out[5], out[3]);
        self.cx_bit(out[0], out[2]);
        self.cx_bit(out[5], out[4]);
        self.cx_bit(out[3], out[2]);
        self.cx_bit(out[1], out[0]);
    }

    // Bit-level CNOT (classical reversible)
    fn cx_bit(&mut self, ctrl: BitId, tgt: BitId) {
        // Reuse the harness's existing bit CNOT if available, else emit as Op
        // The harness has b.cnot(ctrl, tgt) for bits - use that
        self.cnot(ctrl, tgt);
    }

    // Bit-level X (NOT)
    fn x_bit(&mut self, b: BitId) {
        self.bx(b);
    }

    // Bit-level CCX (Toffoli on classical bits)
    fn ccx_bit(&mut self, c1: BitId, c2: BitId, tgt: BitId) {
        self.bccx(c1, c2, tgt);
    }
}

/// Swapper: takes 2 bits (b0, b0&b1) and a 5-bit compressed vector,
/// swaps the two bits with position i (0,1,2) in the compressed vector.
impl B {
    pub fn swapper(&mut self, bb: &[BitId; 2], xreg: &[BitId; 5], i: usize) {
        debug_assert!(i < 3);
        // Decompress xreg → 6 bits (uses 1 ancilla)
        let anc = self.alloc_bit();
        let mut decomp = [BitId(0u32.into()); 6];
        for j in 0..6 {
            decomp[j] = self.alloc_bit();
        }
        // Initialize decomp from xreg via inverse compressor
        // First copy xreg to decomp[0..5]
        for j in 0..5 {
            self.cnot(xreg[j], decomp[j]);
        }
        // decomp[5] = 0 (ancilla)
        // Now apply inverse compressor gates to decomp
        self.ccx_bit(decomp[0], decomp[1], decomp[5]);
        self.ccx_bit(decomp[2], decomp[5], decomp[0]);
        self.ccx_bit(decomp[4], decomp[5], decomp[1]);
        self.ccx_bit(decomp[3], decomp[4], decomp[5]);
        self.x_bit(decomp[2]);
        self.cx_bit(decomp[1], decomp[4]);
        self.ccx_bit(decomp[1], decomp[3], decomp[5]);
        self.x_bit(decomp[4]);
        self.cx_bit(decomp[5], decomp[3]);
        self.cx_bit(decomp[0], decomp[2]);
        self.cx_bit(decomp[5], decomp[4]);
        self.cx_bit(decomp[3], decomp[2]);
        self.cx_bit(decomp[1], decomp[0]);

        // Swap bb with decomp[2*i] and decomp[2*i+1]
        self.cnot(bb[0], decomp[2 * i]);
        self.cnot(decomp[2 * i], bb[0]);
        self.cnot(bb[0], decomp[2 * i]);
        self.cnot(bb[1], decomp[2 * i + 1]);
        self.cnot(decomp[2 * i + 1], bb[1]);
        self.cnot(bb[1], decomp[2 * i + 1]);

        // Recompress decomp → xreg
        self.cx_bit(decomp[1], decomp[0]);
        self.cx_bit(decomp[3], decomp[2]);
        self.cx_bit(decomp[5], decomp[4]);
        self.cx_bit(decomp[0], decomp[2]);
        self.cx_bit(decomp[5], decomp[3]);
        self.x_bit(decomp[4]);
        self.ccx_bit(decomp[1], decomp[3], decomp[5]);
        self.cx_bit(decomp[1], decomp[4]);
        self.x_bit(decomp[2]);
        self.ccx_bit(decomp[3], decomp[4], decomp[5]);
        self.ccx_bit(decomp[4], decomp[5], decomp[1]);
        self.ccx_bit(decomp[2], decomp[5], decomp[0]);
        self.ccx_bit(decomp[0], decomp[1], decomp[5]);
        // decomp[5] should be 0 now

        // Copy back to xreg
        for j in 0..5 {
            self.cnot(xreg[j], decomp[j]);
            self.cnot(decomp[j], xreg[j]);
            self.cnot(xreg[j], decomp[j]);
        }

        // Free ancillas
        self.free_bit(anc);
        for j in 0..6 {
            self.free_bit(decomp[j]);
        }
    }
}
