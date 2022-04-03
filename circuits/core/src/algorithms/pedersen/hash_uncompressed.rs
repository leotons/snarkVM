// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

use std::borrow::Cow;

impl<E: Environment, const NUM_WINDOWS: usize, const WINDOW_SIZE: usize> Pedersen<E, NUM_WINDOWS, WINDOW_SIZE> {
    pub fn hash_uncompressed(&self, input: &[Boolean<E>]) -> Group<E> {
        let constant_false = Boolean::<E>::constant(false);

        let mut input = Cow::Borrowed(input);
        match input.len() <= WINDOW_SIZE * NUM_WINDOWS {
            // Pad the input if it is under the required parameter size.
            true => input.to_mut().resize(WINDOW_SIZE * NUM_WINDOWS, constant_false),
            // Ensure the input size is within the parameter size,
            false => E::halt("incorrect input length for pedersen hash"),
        }

        // Compute sum of h_i^{m_i} for all i.
        input
            .chunks(WINDOW_SIZE)
            .zip_eq(&self.bases)
            .flat_map(|(bits, powers)| {
                bits.iter()
                    .zip_eq(powers)
                    .map(|(bit, base)| Group::<E>::ternary(bit, base, &Group::<E>::zero()))
                    .collect::<Vec<Group<E>>>()
            })
            .fold(Group::<E>::zero(), |acc, x| acc + x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_algorithms::{crh::PedersenCRH, CRH};
    use snarkvm_circuits_environment::Circuit;
    use snarkvm_curves::AffineCurve;
    use snarkvm_utilities::{test_rng, UniformRand};

    const ITERATIONS: usize = 10;
    const MESSAGE: &str = "PedersenCircuit0";
    const WINDOW_SIZE_MULTIPLIER: usize = 8;

    type Projective = <<Circuit as Environment>::Affine as AffineCurve>::Projective;

    fn check_hash_uncompressed<const NUM_WINDOWS: usize, const WINDOW_SIZE: usize>(
        mode: Mode,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        // Initialize the Pedersen hash.
        let native = PedersenCRH::<Projective, NUM_WINDOWS, WINDOW_SIZE>::setup(MESSAGE);
        let circuit = Pedersen::<Circuit, NUM_WINDOWS, WINDOW_SIZE>::setup(MESSAGE);
        // Determine the number of inputs.
        let num_input_bits = NUM_WINDOWS * WINDOW_SIZE;

        for i in 0..ITERATIONS {
            // Sample a random input.
            let input = (0..num_input_bits).map(|_| bool::rand(&mut test_rng())).collect::<Vec<bool>>();
            // Compute the expected hash.
            let expected = native.hash(&input).expect("Failed to hash native input");
            // Prepare the circuit input.
            let circuit_input: Vec<Boolean<_>> = Inject::new(mode, input);

            Circuit::scope(format!("Pedersen {mode} {i}"), || {
                // Perform the hash operation.
                let candidate = circuit.hash_uncompressed(&circuit_input);
                assert_scope!(num_constants, num_public, num_private, num_constraints);
                assert_eq!(expected, candidate.eject_value());
            });
        }
    }

    #[test]
    fn test_hash_uncompressed_constant() {
        // Set the number of windows, and modulate the window size.
        check_hash_uncompressed::<1, WINDOW_SIZE_MULTIPLIER>(Mode::Constant, 32, 0, 0, 0);
        check_hash_uncompressed::<1, { 2 * WINDOW_SIZE_MULTIPLIER }>(Mode::Constant, 64, 0, 0, 0);
        check_hash_uncompressed::<1, { 3 * WINDOW_SIZE_MULTIPLIER }>(Mode::Constant, 96, 0, 0, 0);
        check_hash_uncompressed::<1, { 4 * WINDOW_SIZE_MULTIPLIER }>(Mode::Constant, 128, 0, 0, 0);
        check_hash_uncompressed::<1, { 5 * WINDOW_SIZE_MULTIPLIER }>(Mode::Constant, 160, 0, 0, 0);

        // Set the window size, and modulate the number of windows.
        check_hash_uncompressed::<1, WINDOW_SIZE_MULTIPLIER>(Mode::Constant, 32, 0, 0, 0);
        check_hash_uncompressed::<2, WINDOW_SIZE_MULTIPLIER>(Mode::Constant, 64, 0, 0, 0);
        check_hash_uncompressed::<3, WINDOW_SIZE_MULTIPLIER>(Mode::Constant, 96, 0, 0, 0);
        check_hash_uncompressed::<4, WINDOW_SIZE_MULTIPLIER>(Mode::Constant, 128, 0, 0, 0);
        check_hash_uncompressed::<5, WINDOW_SIZE_MULTIPLIER>(Mode::Constant, 160, 0, 0, 0);
    }

    #[test]
    fn test_hash_uncompressed_public() {
        // Set the number of windows, and modulate the window size.
        check_hash_uncompressed::<1, WINDOW_SIZE_MULTIPLIER>(Mode::Public, 16, 0, 45, 45);
        check_hash_uncompressed::<1, { 2 * WINDOW_SIZE_MULTIPLIER }>(Mode::Public, 32, 0, 93, 93);
        check_hash_uncompressed::<1, { 3 * WINDOW_SIZE_MULTIPLIER }>(Mode::Public, 48, 0, 141, 141);
        check_hash_uncompressed::<1, { 4 * WINDOW_SIZE_MULTIPLIER }>(Mode::Public, 64, 0, 189, 189);
        check_hash_uncompressed::<1, { 5 * WINDOW_SIZE_MULTIPLIER }>(Mode::Public, 80, 0, 237, 237);

        // Set the window size, and modulate the number of windows.
        check_hash_uncompressed::<1, WINDOW_SIZE_MULTIPLIER>(Mode::Public, 16, 0, 45, 45);
        check_hash_uncompressed::<2, WINDOW_SIZE_MULTIPLIER>(Mode::Public, 32, 0, 93, 93);
        check_hash_uncompressed::<3, WINDOW_SIZE_MULTIPLIER>(Mode::Public, 48, 0, 141, 141);
        check_hash_uncompressed::<4, WINDOW_SIZE_MULTIPLIER>(Mode::Public, 64, 0, 189, 189);
        check_hash_uncompressed::<5, WINDOW_SIZE_MULTIPLIER>(Mode::Public, 80, 0, 237, 237);
    }

    #[test]
    fn test_hash_uncompressed_private() {
        // Set the number of windows, and modulate the window size.
        check_hash_uncompressed::<1, WINDOW_SIZE_MULTIPLIER>(Mode::Private, 16, 0, 45, 45);
        check_hash_uncompressed::<1, { 2 * WINDOW_SIZE_MULTIPLIER }>(Mode::Private, 32, 0, 93, 93);
        check_hash_uncompressed::<1, { 3 * WINDOW_SIZE_MULTIPLIER }>(Mode::Private, 48, 0, 141, 141);
        check_hash_uncompressed::<1, { 4 * WINDOW_SIZE_MULTIPLIER }>(Mode::Private, 64, 0, 189, 189);
        check_hash_uncompressed::<1, { 5 * WINDOW_SIZE_MULTIPLIER }>(Mode::Private, 80, 0, 237, 237);

        // Set the window size, and modulate the number of windows.
        check_hash_uncompressed::<1, WINDOW_SIZE_MULTIPLIER>(Mode::Private, 16, 0, 45, 45);
        check_hash_uncompressed::<2, WINDOW_SIZE_MULTIPLIER>(Mode::Private, 32, 0, 93, 93);
        check_hash_uncompressed::<3, WINDOW_SIZE_MULTIPLIER>(Mode::Private, 48, 0, 141, 141);
        check_hash_uncompressed::<4, WINDOW_SIZE_MULTIPLIER>(Mode::Private, 64, 0, 189, 189);
        check_hash_uncompressed::<5, WINDOW_SIZE_MULTIPLIER>(Mode::Private, 80, 0, 237, 237);
    }
}
