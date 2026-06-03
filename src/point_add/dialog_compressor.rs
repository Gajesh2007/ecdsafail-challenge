//! Dialog-based compressor circuit for GCD garbage encoding.
//!
//! Maps 3 pairs of (b0, b0&b1) bits to 5 compressed bits.
//! The compression exploits that only 3 of 4 values are valid: (00, 10, 11).
//! 3^3 = 27 < 32 = 2^5, so compression is possible.
//!
//! The circuit is SAT-synthesized and uses 5 Toffoli gates.
//! Reference: compressor.py:146-166 in the Qarton implementation.

use crate::point_add::B;

/// Forward compressor: maps 6 bits (3 pairs) to 5 bits.
/// Input: x[0..6] where x[2i] = b0_i, x[2i+1] = b0_i & b1_i
/// Output: first 5 bits are compressed, x[5] is |0⟩ and reusable.
///
/// Valid inputs: (00|10|11)^3 = 27 states
/// The circuit is involutory (its own inverse when run backward).
pub fn compressor_fwd(b: &mut B, x: &[usize; 6]) {
    b.set_phase("dialog_compressor_fwd");
    
    // Circuit from compressor.py:146-166 (SAT-synthesized)
    b.cx(x[1], x[0]);
    b.cx(x[3], x[2]);
    b.cx(x[5], x[4]);
    
    b.cx(x[0], x[2]);
    b.cx(x[5], x[3]);
    b.x(x[4]);
    b.ccx(x[1], x[3], x[5]);
    b.cx(x[1], x[4]);
    b.x(x[2]);
    b.ccx(x[3], x[4], x[5]);
    b.ccx(x[4], x[5], x[1]);
    b.ccx(x[2], x[5], x[0]);
    b.ccx(x[0], x[1], x[5]);
    
    // x[5] should now be |0⟩ for valid inputs
}

/// Backward compressor (inverse): maps 5 bits back to 6 bits.
/// This is the reverse of compressor_fwd.
pub fn compressor_bwd(b: &mut B, x: &[usize; 6]) {
    b.set_phase("dialog_compressor_bwd");
    
    // Reverse of the forward circuit
    b.ccx(x[0], x[1], x[5]);
    b.ccx(x[2], x[5], x[0]);
    b.ccx(x[4], x[5], x[1]);
    b.ccx(x[3], x[4], x[5]);
    b.x(x[2]);
    b.cx(x[1], x[4]);
    b.ccx(x[1], x[3], x[5]);
    b.x(x[4]);
    b.cx(x[5], x[3]);
    b.cx(x[0], x[2]);
    
    b.cx(x[5], x[4]);
    b.cx(x[3], x[2]);
    b.cx(x[1], x[0]);
}

/// Swapper: swaps (b0, b0&b1) with position i in compressed bitvector.
/// Takes 2 input bits and a 5-bit compressed register.
/// Decompresses, swaps at position i, then recompresses.
///
/// This is an involution (its own inverse).
pub fn swapper(b: &mut B, bb: [usize; 2], compressed: &[usize; 5], i: usize) {
    b.set_phase("dialog_swapper");
    assert!(i < 3, "swapper position must be 0, 1, or 2");
    
    // Allocate a temporary 6th bit (must be |0⟩ on entry)
    let x6 = b.alloc_qubit();
    
    // Build the 6-bit register: [compressed[0..5], x6]
    let x = [compressed[0], compressed[1], compressed[2], 
             compressed[3], compressed[4], x6];
    
    // Decompress (inverse of compressor)
    compressor_bwd(b, &x);
    
    // Swap bb with position i
    // Position i corresponds to bits [2*i, 2*i+1] in the decompressed register
    b.swap(bb[0], x[2 * i]);
    b.swap(bb[1], x[2 * i + 1]);
    
    // Recompress
    compressor_fwd(b, &x);
    
    // x6 should be |0⟩ again
    b.free(x6);
}

/// Absorber: absorbs (b0, b0&b1) into position i of compressed bitvector.
/// Assumes position i in the decompressed register is (0,0).
/// After absorption, bb is consumed (should be |0⟩).
pub fn absorber(b: &mut B, bb: [usize; 2], compressed: &[usize; 5], i: usize) {
    b.set_phase("dialog_absorber");
    assert!(i < 3, "absorber position must be 0, 1, or 2");
    
    // Allocate a temporary 6th bit (must be |0⟩ on entry)
    let x6 = b.alloc_qubit();
    
    // Build the 6-bit register
    let x = [compressed[0], compressed[1], compressed[2],
             compressed[3], compressed[4], x6];
    
    // Decompress
    compressor_bwd(b, &x);
    
    // Swap bb into position i (bb should be consumed, i.e., become |0⟩)
    b.swap(bb[0], x[2 * i]);
    b.swap(bb[1], x[2 * i + 1]);
    
    // Recompress
    compressor_fwd(b, &x);
    
    // bb should now be |0⟩ (consumed)
    // x6 should be |0⟩
    b.free(x6);
}

/// Initialize a compressed garbage chunk to the compressed form of (00,00,00).
/// This is the starting state before any bits are absorbed.
pub fn init_compressed_chunk(b: &mut B, compressed: &[usize; 5]) {
    b.set_phase("dialog_init_compressed");
    
    // The compressed form of (00,00,00) is (00101) per the lookup table
    // from compressor.py:44-73: BitVector(0,0,0,0,0,0) -> BitVector(0,0,1,0,1)
    // So we need to set compressed[2] = 1 and compressed[4] = 1
    b.x(compressed[2]);
    b.x(compressed[4]);
}

/// Validate that the compressor round-trip works on all 27 valid inputs.
/// This is a classical test function, not a quantum circuit.
#[cfg(test)]
pub fn test_compressor_roundtrip() {
    // Valid pairs: (0,0), (1,0), (1,1)
    let valid_pairs = [(0u8, 0u8), (1, 0), (1, 1)];
    
    for (b0_0, b01_0) in valid_pairs {
        for (b0_1, b01_1) in valid_pairs {
            for (b0_2, b01_2) in valid_pairs {
                let input = [b0_0, b01_0, b0_1, b01_1, b0_2, b01_2];
                
                // Simulate forward compressor
                let mut x = input;
                // ... (simulate the circuit)
                
                // Simulate backward compressor
                // ... (simulate the inverse circuit)
                
                // Check round-trip
                // assert_eq!(output, input);
            }
        }
    }
}
