use core::fmt::Debug;

use serde::{Deserialize, Serialize};

use super::builder::AirBuilder;
use super::instruction::Instruction;
use super::register::bit::BitRegister;
use super::register::memory::MemorySlice;
use super::register::{Register, RegisterSerializable};
use super::trace::writer::TraceWriter;
use super::AirParameters;
use crate::air::parser::AirParser;
use crate::air::AirConstraint;
use crate::math::prelude::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SelectInstruction<T> {
    bit: BitRegister,
    true_value: T,
    false_value: T,
    pub result: T,
}

impl<L: AirParameters> AirBuilder<L> {
    pub fn select<T: Register>(&mut self, bit: &BitRegister, a: &T, b: &T) -> T
    where
        L::Instruction: From<SelectInstruction<T>>,
    {
        let result = self.alloc::<T>();
        let instr = SelectInstruction {
            bit: *bit,
            true_value: *a,
            false_value: *b,
            result,
        };
        self.register_instruction(instr);
        result
    }

    pub fn set_select<T: Register>(&mut self, bit: &BitRegister, a: &T, b: &T, result: &T)
    where
        L::Instruction: From<SelectInstruction<T>>,
    {
        let instr = SelectInstruction {
            bit: *bit,
            true_value: *a,
            false_value: *b,
            result: *result,
        };
        self.register_instruction(instr);
    }
}

impl<AP: AirParser, T: Register> AirConstraint<AP> for SelectInstruction<T> {
    fn eval(&self, parser: &mut AP) {
        let bit = self.bit.eval(parser);
        let true_slice = self.true_value.register().eval_slice(parser).to_vec();
        let false_slice = self.false_value.register().eval_slice(parser).to_vec();
        let result_slice = self.result.register().eval_slice(parser).to_vec();

        let one = parser.one();
        let one_minus_bit = parser.sub(one, bit);

        let constraints = true_slice
            .iter()
            .zip(false_slice.iter())
            .zip(result_slice.iter())
            .map(|((x_true, x_false), x)| {
                let bit_x_true = parser.mul(*x_true, bit);
                let one_minus_bit_x_false = parser.mul(*x_false, one_minus_bit);
                let expected_res = parser.add(bit_x_true, one_minus_bit_x_false);
                parser.sub(expected_res, *x)
            })
            .collect::<Vec<_>>();

        for consr in constraints {
            parser.constraint(consr);
        }
    }
}

impl<F: Field, T: Register + Debug> Instruction<F> for SelectInstruction<T> {
    fn trace_layout(&self) -> Vec<MemorySlice> {
        vec![*self.result.register()]
    }

    fn inputs(&self) -> Vec<MemorySlice> {
        vec![
            *self.bit.register(),
            *self.true_value.register(),
            *self.false_value.register(),
        ]
    }

    fn write(&self, writer: &TraceWriter<F>, row_index: usize) {
        let bit = writer.read(&self.bit, row_index);
        let true_value = writer.read(&self.true_value, row_index);
        let false_value = writer.read(&self.false_value, row_index);

        if bit == F::ONE {
            writer.write(&self.result, &true_value, row_index);
        } else {
            writer.write(&self.result, &false_value, row_index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chip::builder::tests::*;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct SelectorTest;

    impl AirParameters for SelectorTest {
        type Field = GoldilocksField;
        type CubicParams = GoldilocksCubicParameters;

        const NUM_ARITHMETIC_COLUMNS: usize = 0;
        const NUM_FREE_COLUMNS: usize = 4;
        type Instruction = SelectInstruction<BitRegister>;
    }

    // #[test]
    // fn test_selector() {
    //     type F = GoldilocksField;
    //     type L = SelectorTest;
    //     type SC = PoseidonGoldilocksStarkConfig;

    //     let mut builder = AirBuilder::<L>::new();

    //     let bit = builder.alloc::<BitRegister>();
    //     let x = builder.alloc::<BitRegister>();
    //     let y = builder.alloc::<BitRegister>();

    //     let z = builder.select(&bit, &x, &y);

    //     let (air, trace_data) = builder.build();

    //     let generator = ArithmeticGenerator::<L>::new(trace_data, num_rows);

    //     let (tx, rx) = channel();
    //     for i in 0..num_rows {
    //         let writer = generator.new_writer();
    //         let handle = tx.clone();
    //         let x_i = F::from_canonical_u16(0u16);
    //         let y_i = F::from_canonical_u16(1u16);
    //         let bit_i = i % 2 == 0;
    //         let fbit = if bit_i { F::ONE } else { F::ZERO };
    //         rayon::spawn(|| {
    //             writer.write(&bit, &fbit, i);
    //             writer.write(&x, &x_i, i);
    //             writer.write(&y, &y_i, i);
    //             writer.write_row_instructions(&generator.air_data, i);

    //             handle.send(1).unwrap();
    //         });
    //     }
    //     drop(tx);
    //     for msg in rx.iter() {
    //         assert!(msg == 1);
    //     }
    //     let stark = Starky::new(air);
    //     let config = SC::standard_fast_config(num_rows);

    //     // Generate proof and verify as a stark
    //     test_starky(&stark, &config, &generator, &[]);

    //     // Test the recursive proof.
    //     test_recursive_starky(stark, config, generator, &[]);
    // }
}
