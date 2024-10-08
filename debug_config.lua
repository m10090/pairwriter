local dap = require('dap')


dap.adapters.rust  = {
  type = 'executable',
  command = 'lldb-vscode',
  name = "lldb"
}

dap.configurations.rust = {
  {
    name = "Launch Rust Program",
    type = "codelldb", -- matches the adapter
    request = "launch",
    program = function()
      -- Prompt for the path to the executable to be debugged
      return vim.fn.input('Path to executable: ', vim.fn.getcwd() .. '/target/debug/', 'file')
    end,
    args =  {"server" ,"8080"},
    cwd = '${workspaceFolder}',
    stopOnEntry = false,
    runInTerminal = false,
  },
}
