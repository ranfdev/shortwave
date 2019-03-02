hook_conf = configuration_data()
hook_conf.set_quoted('CARGO_HOME', join_paths(meson.build_root(), 'target', 'cargo-home'))
hook = configure_file(
  input: 'pre-commit.in',
  output: 'pre-commit',
  configuration: hook_conf
)

message('Setting up pre-commit hook...')
run_command('cp', '-f',
  join_paths(meson.build_root(), 'hooks', 'pre-commit'),
  join_paths(meson.source_root(), '.git', 'hooks', 'pre-commit')
)

message('Making pre-commit hook executable...')
run_command('chmod', '+x',
  join_paths(meson.source_root(), '.git', 'hooks', 'pre-commit')
)

