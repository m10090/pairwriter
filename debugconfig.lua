-- this file is for debuging configuration in nvim dap
-- just :so % to load this file
require("dap").configurations.rust = {
	{
		type = "codelldb",
		request = "launch",
		name = "Launch Rust Program",
		program = function()
			return vim.fn.getenv("PROJECT_DIR") .. "/target/debug/shared-programming"
		end,
		args = { "client", "127.0.0.1", "rania", "--features" }, -- change as needed
		stopOnEntry = false,
	},
}
