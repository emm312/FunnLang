mod buffer;

mod token;
mod scanner;

mod errors;

mod lexer;

mod preprocess;

mod parser;
mod ast;

mod compiler;

use std::process::exit;

use codegem::regalloc::RegAlloc;
use codegem::arch::{urcl::UrclSelector, rv64::RvSelector, x64::X64Selector};
use scanner::*;
use lexer::*;
use preprocess::*;
use ast::*;

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(value_name = "Input file")]
    input_file: String,

    #[arg(short, long, default_value="urcl")]
    target: String,

    #[arg(short, long, default_value="out.s")]
    output: String
}

pub enum CompilerTarget {
    URCL,
    RV64,
    X64,
}

impl CompilerTarget {
    pub fn from_string(str: &str) -> Option<Self> {
        use CompilerTarget::*;
        match str.to_ascii_lowercase().as_str() {
            "urcl" => {
                Some(URCL)
            }
            "rv64" | "riscv" => {
                Some(RV64)
            }
            "x86" | "x86_64" | "x64" => {
                Some(X64)
            }
            _ => {
                None
            }
        }
    }
    pub fn to_str(&self) -> &str {
        use CompilerTarget::*;
        match self {
            URCL => "urcl",
            RV64 => "rv64",
            X64  => "x64"
        }
    }
}

fn main() {
    let args = Args::parse();
    let target = match CompilerTarget::from_string(&args.target) {
        Some(v) => v,
        None => {
            println!("\x1b[1;31merror:\x1b[0m unsupported target '{}' had been specified\x1b[0m", args.target);
            exit(1)
        }
    };
    
    let src = std::fs::read_to_string(args.input_file.clone()).expect("F");
    
    let mut lex = lex(&mut Scanner::new(src.chars().collect::<Vec<char>>()), 0);
    let mut srcs = vec![src];
    let mut files = vec![args.input_file];
    let tok = preprocess(&mut lex, &mut srcs, &mut files, &mut 1, &target);
    let ast = generate_ast(&tok);
    if ast.err.errors.len() == 0 {
        println!("{:#?}", ast.ast);
    } else {
        println!("{}", ast.err.as_string(srcs, files));
        return
    }
    let mut file = std::fs::File::create(args.output).unwrap();
    let ir = compiler::ast_compiler::compiler(ast.ast);
    println!("{}", ir);

    use CompilerTarget::*;
    match target {
            URCL => {
            let mut vcode = ir.lower_to_vcode::<_, UrclSelector>();
            asm_gen(&mut vcode, &mut file)
        },
        RV64 => {
            let mut vcode = ir.lower_to_vcode::<_, RvSelector>();
            asm_gen(&mut vcode, &mut file)
        },
        X64  => {
            let mut vcode = ir.lower_to_vcode::<_, X64Selector>();
            asm_gen(&mut vcode, &mut file)
        }
    }
}
pub fn to_mut_ptr<T>(a: &T) -> &mut T {
    unsafe {
        &mut *(a as *const T as *mut T)
    }
}
pub fn asm_gen<T: codegem::arch::Instr>(vcode: &mut codegem::arch::VCode<T>, file: &mut std::fs::File) {
    vcode.allocate_regs::<RegAlloc>();
    match vcode.emit_assembly(file) {
        Ok(_) => (),
        Err(err) => {
            println!("\x1b[1;31merror: error while emitting assembly code, reason: {}.\x1b[0m", err);
            exit(1)
        }
    }
}