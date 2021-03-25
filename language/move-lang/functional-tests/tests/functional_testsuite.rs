// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use diem_types::account_address::AccountAddress as DiemAddress;
use functional_tests::{
    compiler::{Compiler, ScriptOrModule},
    testsuite,
};
use move_lang::{
    self, command_line::read_bool_env_var, compiled_unit::CompiledUnit, errors, shared::Address,
    FullyCompiledProgram,
};
use once_cell::sync::Lazy;
use std::{convert::TryFrom, fmt, io::Write, path::Path};
use tempfile::NamedTempFile;

pub const STD_LIB_DIR: &str = "../../diem-framework/modules";
pub const FUNCTIONAL_TEST_DIR: &str = "tests";

struct MoveSourceCompiler<'a> {
    pre_compiled_deps: &'a FullyCompiledProgram,
    deps: Vec<String>,
    temp_files: Vec<NamedTempFile>,
}

impl<'a> MoveSourceCompiler<'a> {
    fn new(pre_compiled_deps: &'a FullyCompiledProgram) -> Self {
        MoveSourceCompiler {
            pre_compiled_deps,
            deps: vec![],
            temp_files: vec![],
        }
    }

    fn move_compile_with_stdlib(
        &self,
        targets: &[String],
        sender: Option<Address>,
    ) -> anyhow::Result<(
        errors::FilesSourceText,
        Result<Vec<CompiledUnit>, errors::Errors>,
    )> {
        let (files, pprog_and_comments_res) =
            move_lang::move_parse(targets, &self.deps, sender, None)?;
        let (_comments, sender_opt, pprog) = match pprog_and_comments_res {
            Err(errors) => return Ok((files, Err(errors))),
            Ok(res) => res,
        };

        let result = match move_lang::move_continue_up_to(
            Some(&self.pre_compiled_deps),
            move_lang::PassResult::Parser(sender_opt, pprog),
            move_lang::Pass::Compilation,
        ) {
            Ok(move_lang::PassResult::Compilation(units)) => Ok(units),
            Ok(_) => unreachable!(),
            Err(errors) => Err(errors),
        };
        Ok((files, result))
    }
}

#[derive(Debug)]
struct MoveSourceCompilerError(pub String);

impl fmt::Display for MoveSourceCompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n\n{}", self.0)
    }
}

impl std::error::Error for MoveSourceCompilerError {}

impl<'a> Compiler for MoveSourceCompiler<'a> {
    /// Compile a transaction script or module.
    fn compile<Logger: FnMut(String)>(
        &mut self,
        _log: Logger,
        address: DiemAddress,
        input: &str,
    ) -> Result<ScriptOrModule> {
        let cur_file = NamedTempFile::new()?;
        let sender_addr = Address::try_from(address.as_ref()).unwrap();
        cur_file.reopen()?.write_all(input.as_bytes())?;
        let cur_path = cur_file.path().to_str().unwrap().to_owned();

        let targets = &vec![cur_path.clone()];
        let sender = Some(sender_addr);

        let (files, units_or_errors) = self.move_compile_with_stdlib(targets, sender)?;
        let unit = match units_or_errors {
            Err(errors) => {
                let error_buffer = if read_bool_env_var(testsuite::PRETTY) {
                    move_lang::errors::report_errors_to_color_buffer(files, errors)
                } else {
                    move_lang::errors::report_errors_to_buffer(files, errors)
                };
                return Err(
                    MoveSourceCompilerError(String::from_utf8(error_buffer).unwrap()).into(),
                );
            }
            Ok(mut units) => {
                let len = units.len();
                if len != 1 {
                    bail!("Invalid input. Expected 1 compiled unit but got {}", len)
                }
                units.pop().unwrap()
            }
        };

        Ok(match unit {
            CompiledUnit::Script { script, .. } => ScriptOrModule::Script(None, script),
            CompiledUnit::Module { module, .. } => {
                let input = if input.starts_with("address") {
                    input.to_string()
                } else {
                    format!("address {} {{\n{}\n}}", sender_addr, input)
                };
                cur_file.reopen()?.write_all(input.as_bytes())?;
                self.temp_files.push(cur_file);
                self.deps.push(cur_path);
                ScriptOrModule::Module(module)
            }
        })
    }

    fn use_compiled_genesis(&self) -> bool {
        false
    }
}

static DIEM_PRECOMPILED_STDLIB: Lazy<FullyCompiledProgram> = Lazy::new(|| {
    let (files, program_res) = move_lang::move_construct_pre_compiled_lib(
        &diem_framework::diem_stdlib_files(),
        None,
        None,
        false,
    )
    .unwrap();
    match program_res {
        Ok(stdlib) => stdlib,
        Err(errors) => {
            eprintln!("!!!Standard library failed to compile!!!");
            errors::report_errors(files, errors)
        }
    }
});

fn functional_testsuite(path: &Path) -> datatest_stable::Result<()> {
    testsuite::functional_tests(MoveSourceCompiler::new(&*DIEM_PRECOMPILED_STDLIB), path)
}

datatest_stable::harness!(functional_testsuite, FUNCTIONAL_TEST_DIR, r".*\.move$");
