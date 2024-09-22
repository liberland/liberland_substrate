pub struct MockedLiberlandExtensionSuccess;
impl ink::env::test::ChainExtension for MockedLiberlandExtensionSuccess {
	fn ext_id(&self) -> u16 {
		0
	}

	fn call(&mut self, _func_id: u16, _input: &[u8], _output: &mut Vec<u8>) -> u32 {
		0
	}
}

pub struct MockedLiberlandExtensionFail;
impl ink::env::test::ChainExtension for MockedLiberlandExtensionFail {
	fn ext_id(&self) -> u16 {
		0
	}

	fn call(&mut self, _func_id: u16, _input: &[u8], _output: &mut Vec<u8>) -> u32 {
		1
	}
}
