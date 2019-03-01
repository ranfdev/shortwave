global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', join_paths(shortwave_prefix, shortwave_pkgdatadir))
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('NAME_PREFIX', name_prefix)
global_conf.set_quoted('NAME', application_name)
global_conf.set_quoted('VERSION', shortwave_version + version_suffix)
config = configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: global_conf
)

c = run_command(grabber)
sources = c.stdout().strip().split('\n')

# Copy the configuration into src for Cargo
run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)

cargo_release = custom_target('cargo-build',
                        build_by_default: true,
                        input: [sources],
                        output: ['shortwave'],
                        console: true,
                        install: true,
                        install_dir: shortwave_bindir,
                        command: [cargo_script,
                        	  meson.build_root(),
                        	  meson.source_root(),
                        	  '@OUTPUT@',
                        	  shortwave_localedir,
                        	  profile])

