# Adatped from GNOME Social by Christopher Davis (GPLv3)
# https://gitlab.gnome.org/BrainBlasted/Social/blob/7e8f36b5e1b96fd64d152de995e15171a823836c/meson.build

project(
  'shortwave', 'rust',
  version: '7.99.0',
  license: 'GPLv3',
)

shortwave_version = meson.project_version()
version_array = shortwave_version.split('.')
shortwave_major_version = version_array[0].to_int()
shortwave_minor_version = version_array[1].to_int()
shortwave_version_micro = version_array[2].to_int()

shortwave_prefix = get_option('prefix')
shortwave_bindir = join_paths(shortwave_prefix, get_option('bindir'))
shortwave_localedir = join_paths(shortwave_prefix, get_option('localedir'))

shortwave_datadir = get_option('datadir')
shortwave_pkgdatadir = join_paths(shortwave_datadir, meson.project_name())
shortwave_icondir = join_paths(shortwave_datadir, 'icons')

if get_option('profile') == 'development'
  profile = 'Devel'
  name_prefix = '(Development) '
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format (vcs_tag)
  endif
else
  profile = ''
  name_prefix = ''
  version_suffix = ''
endif

application_id = 'de.haeckerfelix.Shortwave@0@'.format(profile)
application_name = 'Shortwave'

i18n = import('i18n')
gnome = import('gnome')
podir = join_paths (meson.source_root (), 'po')

subdir('po')
subdir('data')

cargo = find_program('cargo', required: false)
gresource = find_program('glib-compile-resources', required: false)
cargo_vendor = find_program('cargo-vendor', required: false)
cargo_script = find_program('build-aux/cargo.sh')
grabber = find_program('build-aux/grabber.sh')

subdir('src')

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  join_paths(meson.build_root(), 'meson-dist', meson.project_name() + '-' + shortwave_version),
  meson.source_root()
)

meson.add_install_script('build-aux/meson_post_install.py')
