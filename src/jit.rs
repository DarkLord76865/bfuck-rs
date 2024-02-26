use cranelift::prelude::*;
use cranelift::codegen::{ir, verify_function};
use target_lexicon::Triple;
use std::process::exit;
use std::mem;
use memmap2::MmapOptions;
use crate::code::{STORAGE_SIZE, Token, TokenStream};
use crate::io::{getchar, putchar};


pub fn jit(token_stream: TokenStream) {
    let mut flag_builder = settings::builder();
    flag_builder.set("opt_level", "speed_and_size").unwrap();
    let flags = settings::Flags::new(flag_builder);
    let target_isa = match isa::lookup(Triple::host()) {
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
        Err(_) => {
            eprintln!("Unsupported target!");
            exit(1);
        },
    };

    let ptr_type = target_isa.pointer_type();
    let call_conv = isa::CallConv::triple_default(target_isa.triple());

    let mut signature = Signature::new(call_conv);
    signature.params.push(AbiParam::new(ptr_type));
    let mut function = ir::Function::with_name_signature(ir::UserFuncName::default(), signature);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut function, &mut func_ctx);

    let mem_flags = MemFlags::new();

    // start
    let first_block = builder.create_block();
    builder.seal_block(first_block);
    builder.append_block_params_for_function_params(first_block);
    builder.switch_to_block(first_block);

    let data_ptr = Variable::new(0);
    builder.declare_var(data_ptr, ptr_type);

    let memory_address = builder.block_params(first_block)[0];

    let zero = builder.ins().iconst(ptr_type, 0);
    builder.def_var(data_ptr, zero);

    // write and read

    let (read_sig, read_address) = {
        let mut read_sig = Signature::new(call_conv);
        read_sig.returns.push(AbiParam::new(types::I8));
        let read_sig = builder.import_signature(read_sig);

        let read_address = getchar as *const () as i64;
        let read_address = builder.ins().iconst(ptr_type, read_address);
        
        (read_sig, read_address)
    };
    
    let (write_sig, write_address) = {
        let mut write_sig = Signature::new(call_conv);
        write_sig.params.push(AbiParam::new(types::I8));
        let write_sig = builder.import_signature(write_sig);

        let write_address = putchar as *const () as i64;
        let write_address = builder.ins().iconst(ptr_type, write_address);
        
        (write_sig, write_address)
    };


    let mut stack = Vec::new();

    for token in token_stream {
        match token {
            Token::Add(n) => {
                let ptr_val = builder.use_var(data_ptr);
                let cell_address = builder.ins().iadd(memory_address, ptr_val);

                let cell_value = builder.ins().load(types::I8, mem_flags, cell_address, 0);
                let cell_value = builder.ins().iadd_imm(cell_value, n as i64);

                builder.ins().store(mem_flags, cell_value, cell_address, 0);
            },
            Token::Mov(n) => {
                let ptr_val = builder.use_var(data_ptr);
                let ptr_plus = builder.ins().iadd_imm(ptr_val, n as i64);
                
                let ptr_val = {
                    let wrapped = builder.ins().iadd_imm(ptr_val, n as i64 - STORAGE_SIZE as i64);
                    let cmp = builder.ins().icmp_imm(IntCC::SignedLessThan, ptr_plus, STORAGE_SIZE as i64);
                    builder.ins().select(cmp, ptr_plus, wrapped)
                };
                
                builder.def_var(data_ptr, ptr_val);
            },
            Token::Input => {
                let ptr_val = builder.use_var(data_ptr);
                let cell_address = builder.ins().iadd(memory_address, ptr_val);

                let read_res = builder
                    .ins()
                    .call_indirect(read_sig, read_address, &[]);
                let read_res = builder.inst_results(read_res)[0];

                builder.ins().store(mem_flags, read_res, cell_address, 0);
            },
            Token::Output => {
                let ptr_val = builder.use_var(data_ptr);
                let cell_address = builder.ins().iadd(memory_address, ptr_val);
                let cell_value = builder.ins().load(types::I8, mem_flags, cell_address, 0);

                builder.ins().call_indirect(write_sig, write_address, &[cell_value]);
            },
            Token::OpenBr => {
                let inner_block = builder.create_block();
                let after_block = builder.create_block();

                let ptr_val = builder.use_var(data_ptr);
                let cell_address = builder.ins().iadd(memory_address, ptr_val);
                let cell_value = builder.ins().load(types::I8, mem_flags, cell_address, 0);

                let eq_zero_cmp = builder.ins().icmp_imm(IntCC::Equal, cell_value, 0);
                builder.ins().brif(eq_zero_cmp, after_block, &[], inner_block, &[]);

                builder.switch_to_block(inner_block);

                stack.push((inner_block, after_block));
            },
            Token::CloseBr => {
                let (inner_block, after_block) = stack.pop().unwrap();

                let ptr_val = builder.use_var(data_ptr);
                let cell_address = builder.ins().iadd(memory_address, ptr_val);
                let cell_value = builder.ins().load(types::I8, mem_flags, cell_address, 0);
                
                let eq_zero_cmp = builder.ins().icmp_imm(IntCC::Equal, cell_value, 0);
                builder.ins().brif(eq_zero_cmp, after_block, &[], inner_block, &[]);

                builder.seal_block(inner_block);
                builder.seal_block(after_block);

                builder.switch_to_block(after_block);
            },
        }
    }
    
    builder.ins().return_(&[]);
    builder.finalize();

    // end

    if let Err(err) =  verify_function(&function, &*target_isa) {
        eprintln!("{}", err);
        exit(1);
    }

    let mut code_buffer = Vec::new();
    codegen::Context::for_function(function)
        .compile_and_emit(&*target_isa, &mut code_buffer, &mut codegen::control::ControlPlane::default())
        .unwrap();

    let mut buffer = MmapOptions::new()
        .len(code_buffer.len())
        .map_anon()
        .unwrap();
    buffer.copy_from_slice(&code_buffer);
    drop(code_buffer);
    let buffer = buffer.make_exec().unwrap();

    unsafe {
        let memory = [0_u8; STORAGE_SIZE];
        let memory_address = memory.as_ptr();
        let code_fn: unsafe extern "C" fn(*const u8) = mem::transmute(buffer.as_ptr());
        code_fn(memory_address)
    };
}
