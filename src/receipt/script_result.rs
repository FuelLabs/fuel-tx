use fuel_asm::{Instruction, Opcode, PanicReason};
use fuel_types::Word;

use std::mem;

const WORD_SIZE: usize = mem::size_of::<Word>();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-types", derive(serde::Serialize, serde::Deserialize))]
pub struct ScriptResult {
    result: PanicReason,
    instruction: Instruction,
}

impl ScriptResult {
    pub const fn new(result: PanicReason, instruction: Instruction) -> Self {
        Self {
            result,
            instruction,
        }
    }

    pub const fn result(&self) -> &PanicReason {
        &self.result
    }

    pub const fn instruction(&self) -> &Instruction {
        &self.instruction
    }
}

const RESULT_OFFSET: Word = (WORD_SIZE * 8 - 8) as Word;
const INSTR_OFFSET: Word = ((WORD_SIZE - mem::size_of::<u32>()) * 8 - 8) as Word;

impl From<ScriptResult> for Word {
    fn from(r: ScriptResult) -> Word {
        let result = Word::from(r.result);
        let instruction = u32::from(r.instruction) as Word;

        (result << RESULT_OFFSET) | (instruction << INSTR_OFFSET)
    }
}

impl From<Word> for ScriptResult {
    fn from(val: Word) -> Self {
        let result = PanicReason::from(val >> RESULT_OFFSET);
        let instruction = Instruction::from((val >> INSTR_OFFSET) as u32);

        Self::new(result, instruction)
    }
}

impl From<ScriptResult> for Instruction {
    fn from(r: ScriptResult) -> Self {
        r.instruction
    }
}

impl From<ScriptResult> for Opcode {
    fn from(r: ScriptResult) -> Self {
        r.instruction.into()
    }
}

impl From<ScriptResult> for PanicReason {
    fn from(r: ScriptResult) -> Self {
        r.result
    }
}
