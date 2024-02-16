use std::{fs, path::PathBuf, process::exit};

use anyhow::Result;
use clap::Parser;
use target_lexicon::Triple;

use crate::{
    codegen::{jit::JitGenerator, simple::SimpleCompiler},
    lexer::Lexer,
    tokenizer::{cursor::Cursor, Tokenizer},
};

use super::Command;

#[derive(Debug, Clone, Parser)]
pub struct RunCommand {
    /// The path to the file to compile.
    pub file: PathBuf,

    /// Instead of running, dump the tokens.
    #[arg(long = "dump-tokens")]
    pub dump_tokens: bool,

    /// Instead of running, dump the AST.
    #[arg(long = "dump-ast")]
    pub dump_ast: bool,
}

impl Command for RunCommand {
    fn execute(&mut self) -> Result<()> {
        let content = fs::read_to_string(self.file.clone())?;

        let cursor = Cursor::new(
            self.file.clone().to_str().unwrap().to_string(),
            content.chars().collect(),
        );

        let mut tokenizer = Tokenizer::new(cursor.clone());
        let tokens = tokenizer.tokenize();

        if self.dump_tokens {
            println!("{:#?}", tokens);
            return Ok(());
        }

        let mut lexer = Lexer::new(cursor, tokens);
        let exprs = lexer.lex()?;

        if self.dump_ast {
            println!("{:#?}", exprs);
            return Ok(());
        }

        let mut compiler = SimpleCompiler::<JitGenerator>::new(Triple::host())?;

        compiler.compile(exprs)?;

        exit(compiler.run()?);
    }
}
