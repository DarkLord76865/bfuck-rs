//! The Brainfuck JIT-compiler.



use std::mem;

use cranelift::codegen::{ir, verify_function};
use cranelift::prelude::*;
use memmap2::MmapOptions;
use target_lexicon::Triple;

use crate::code::{STORAGE_SIZE, Token, TokenStream};
use crate::error::Error;
use crate::io::{getchar, putchar};



/// JIT-compile and run provided token stream.
/// # Arguments
/// * token_stream - The [TokenStream] to compile.
/// # Returns
/// * `()` - If [Ok].
/// * [Error] - The encountered error, if [Err].
/// # Errors
/// * `UnsupportedPlatformJIT` - The current platform is not supported for JIT-compilation, use interpreter instead.
/// # Example
/// ```
/// use bfuck::{process_code, jit};
///
/// // brainfuck code that prints "Brainfuck"
/// let bf_code = "
/// >++++++[<+++++++++++>-]>+++++++[<+++++++
/// +++++++>-]<->>+++++++++[<+++++++++++>-]>
/// ++++++++[<+++++++++++++>-]<-->>+++++++++
/// +[<+++++++++++>-]<----->>++++++++[<+++++
/// ++++++++>-]<+++>>++++++++++[<+++++++++++
/// >-]>++++++++++[<++++++++++++>-]<------>>
/// +++++++++[<+++++++++++++>-]<<<<<<<<<.>>>
/// >>>>.<<<<<<.>>>.>>.<<<.>>>>>.<<<<<<.>>>.
/// ";
///
/// jit(process_code(bf_code).unwrap()).expect("Unsupported platform.");
/// ```
pub fn jit(token_stream: TokenStream) -> Result<(), Error> {
    // set compilation flags
    let mut flag_builder = settings::builder();
    flag_builder.set("opt_level", "speed_and_size").unwrap();
    let flags = settings::Flags::new(flag_builder);

    // set target ISA
    let target_isa = match isa::lookup(Triple::host()) {
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
        Err(_) => return Err(Error::UnsupportedPlatformJIT),
    };

    // find target pointer type
    let ptr_type = target_isa.pointer_type();

    // find target call convention
    let call_conv = isa::CallConv::triple_default(target_isa.triple());

    // create JIT function with a signature
    // function accepts one parameter - pointer to array of STORAGE_SIZE length and filled with zero bytes
    let mut signature = Signature::new(call_conv);
    signature.params.push(AbiParam::new(ptr_type));
    let mut function = ir::Function::with_name_signature(ir::UserFuncName::default(), signature);

    // create function builder
    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut function, &mut func_ctx);

    // create memory flags (needed for load and store instructions)
    let mem_flags = MemFlags::new();

    // START of building the JIT function
    
    // define initial block
    let first_block = builder.create_block();
    builder.seal_block(first_block);
    builder.append_block_params_for_function_params(first_block);
    builder.switch_to_block(first_block);

    // declare data pointer variable and initialize it with zero
    let data_ptr = Variable::new(0);
    builder.declare_var(data_ptr, ptr_type);
    let zero = builder.ins().iconst(ptr_type, 0);
    builder.def_var(data_ptr, zero);

    // get the memory address of the start of the array (received as a parameter to the function)
    let memory_address = builder.block_params(first_block)[0];

    // input and output functionality is achieved by calling external functions getchar and putchar (defined in io module)

    // declare signature for read function (getchar)
    let mut read_sig = Signature::new(call_conv);
    read_sig.returns.push(AbiParam::new(types::I8));
    let read_sig = builder.import_signature(read_sig);

    // declare address of the read function (getchar)
    let read_address = builder.ins().iconst(ptr_type, getchar as *const () as i64);

    // declare signature for write function (putchar)
    let mut write_sig = Signature::new(call_conv);
    write_sig.params.push(AbiParam::new(types::I8));
    let write_sig = builder.import_signature(write_sig);

    // declare address of the write function (putchar)
    let write_address = builder.ins().iconst(ptr_type, putchar as *const () as i64);

    // stack for tracking loop blocks
    let mut stack = Vec::new();

    // iterate over tokens and generate code for each token
    for token in token_stream {
        match token {
            Token::Add(n) => {
                // load the data pointer value
                let ptr_val = builder.use_var(data_ptr);
                // calculate cell address (memory_address + data_ptr)
                let cell_address = builder.ins().iadd(memory_address, ptr_val);

                // load the value from the current cell (in array)
                let cell_value = builder.ins().load(types::I8, mem_flags, cell_address, 0);
                // add n to the value
                let cell_value = builder.ins().iadd_imm(cell_value, n as i64);

                // store the new value back to the cell
                builder.ins().store(mem_flags, cell_value, cell_address, 0);
            },
            Token::Mov(n) => {
                // load the data pointer value
                let ptr_val = builder.use_var(data_ptr);

                // the new pointer value is == (old_value + n) % STORAGE_SIZE
                // but since remainder operation is expensive, we can calculate
                // both (old_value + n) and (old_value + n - STORAGE_SIZE) and then
                // select the correct value based on the condition (old_value + n < STORAGE_SIZE)

                // old_value + n
                let ptr_plus = builder.ins().iadd_imm(ptr_val, n as i64);
                // old_value + n - STORAGE_SIZE
                let ptr_wrapped = builder.ins().iadd_imm(ptr_val, n as i64 - STORAGE_SIZE as i64);

                // compare (old_value + n) with STORAGE_SIZE
                let cmp = builder.ins().icmp_imm(IntCC::SignedLessThan, ptr_plus, STORAGE_SIZE as i64);

                // select the correct value based on the condition
                let ptr_val = builder.ins().select(cmp, ptr_plus, ptr_wrapped);

                // store the new data pointer value
                builder.def_var(data_ptr, ptr_val);
            },
            Token::Input => {
                // load the data pointer value
                let ptr_val = builder.use_var(data_ptr);
                // calculate cell address (memory_address + data_ptr)
                let cell_address = builder.ins().iadd(memory_address, ptr_val);

                // call the read function (getchar)
                let read_res = builder
                    .ins()
                    .call_indirect(read_sig, read_address, &[]);
                // get the result of the read function
                let read_res = builder.inst_results(read_res)[0];

                // store the read value to the cell
                builder.ins().store(mem_flags, read_res, cell_address, 0);
            },
            Token::Output => {
                // load the data pointer value
                let ptr_val = builder.use_var(data_ptr);
                // calculate cell address (memory_address + data_ptr)
                let cell_address = builder.ins().iadd(memory_address, ptr_val);
                // load the value from the cell
                let cell_value = builder.ins().load(types::I8, mem_flags, cell_address, 0);

                // call the write function (putchar) with the value from the cell
                builder.ins().call_indirect(write_sig, write_address, &[cell_value]);
            },
            Token::OpenBr => {
                // create two new blocks - one for the loop body and one for the code after the loop
                let inner_block = builder.create_block();
                let after_block = builder.create_block();

                // load the data pointer value
                let ptr_val = builder.use_var(data_ptr);
                // calculate cell address (memory_address + data_ptr)
                let cell_address = builder.ins().iadd(memory_address, ptr_val);
                // load the value from the cell
                let cell_value = builder.ins().load(types::I8, mem_flags, cell_address, 0);

                // compare the value from the cell with zero
                // if the value is zero, jump to the block after the loop, otherwise jump to the loop body
                let eq_zero_cmp = builder.ins().icmp_imm(IntCC::Equal, cell_value, 0);
                builder.ins().brif(eq_zero_cmp, after_block, &[], inner_block, &[]);

                // switch to the loop body block (because next command will be inside the loop body)
                builder.switch_to_block(inner_block);

                // push the loop blocks to the stack
                stack.push((inner_block, after_block));
            },
            Token::CloseBr => {
                // get the loop blocks from the stack (guaranteed to be there because loops are checked for correctness in the parser)
                let (inner_block, after_block) = stack.pop().unwrap();

                // load the data pointer value
                let ptr_val = builder.use_var(data_ptr);
                // calculate cell address (memory_address + data_ptr)
                let cell_address = builder.ins().iadd(memory_address, ptr_val);
                // load the value from the cell
                let cell_value = builder.ins().load(types::I8, mem_flags, cell_address, 0);

                // compare the value from the cell with zero
                // if the value is zero, jump to the block after the loop, otherwise jump to the loop body (next iteration)
                let eq_zero_cmp = builder.ins().icmp_imm(IntCC::Equal, cell_value, 0);
                builder.ins().brif(eq_zero_cmp, after_block, &[], inner_block, &[]);

                // now all jumps to these blocks are defined, so we can seal them
                builder.seal_block(inner_block);
                builder.seal_block(after_block);

                // switch to the block after the loop (where next command will be)
                builder.switch_to_block(after_block);
            },
        }
    }

    // return instruction to the end of the function
    builder.ins().return_(&[]);

    // finalize the function
    builder.finalize();
    
    // END of building the JIT function

    // Verify that the function is correct before compiling.
    // This shouldn't fail if we correctly wrote a code for generating the function (which we did).
    assert_eq!(verify_function(&function, &*target_isa), Ok(()), "The JIT function is not valid!");

    // Compile the function to machine code.
    // Shouldn't fail since we verified the function.
    let mut compiled_code = Vec::new();
    codegen::Context::for_function(function)
        .compile_and_emit(&*target_isa, &mut compiled_code, &mut codegen::control::ControlPlane::default())
        .unwrap();

    // Map the compiled code into memory.
    let mut code_buffer = MmapOptions::new()
        .len(compiled_code.len())
        .map_anon()
        .unwrap();
    code_buffer.copy_from_slice(&compiled_code);
    let code_buffer = code_buffer.make_exec().unwrap();
    drop(compiled_code);

    // Execute the JIT function.
    unsafe {
        let memory = [0_u8; STORAGE_SIZE];
        let code_fn: unsafe extern "C" fn(*const u8) = mem::transmute(code_buffer.as_ptr());
        code_fn(memory.as_ptr())
    };

    // Return success after executing the JIT function.
    Ok(())
}
