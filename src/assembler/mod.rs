mod tokenizer;

#[derive(Debug)]
pub struct Assembler<'a> {
    text: &'a str,
}
