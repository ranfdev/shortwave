subdir('icons')

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = i18n.merge_file (
  type: 'desktop',
  input: configure_file(
    input: files('de.haeckerfelix.Shortwave.desktop.in.in'),
    output: 'de.haeckerfelix.Shortwave.desktop.in',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: join_paths (shortwave_datadir, 'applications')
)
# Validate Desktop file
desktop_file_validate = find_program('desktop-file-validate', required: false)
if desktop_file_validate.found()
  test(
    'validate-desktop',
    desktop_file_validate,
    args: [
      desktop_file.full_path()
    ]
  )
endif

# Appdata
appdata_conf = configuration_data()
appdata_conf.set('appid', application_id)
appdata_file = i18n.merge_file (
  input: configure_file(
    input: files('de.haeckerfelix.Shortwave.appdata.xml.in.in'),
    output: 'de.haeckerfelix.Shortwave.appdata.xml.in',
    configuration: appdata_conf
  ),
  output: '@0@.appdata.xml'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: join_paths (shortwave_datadir, 'metainfo')
)
# Validate Appdata
appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test(
    'validate-appdata', appstream_util,
    args: [
      'validate-relax', '--nonet', appdata_file.full_path()
    ]
  )
endif

# dbus service
service_conf = configuration_data()
service_conf.set('appid', application_id)
service_conf.set('bindir', shortwave_bindir)
configure_file(
  input: 'de.haeckerfelix.Shortwave.service.in',
  output: '@0@.service'.format(application_id),
  configuration: service_conf,
  install_dir: join_paths(shortwave_datadir,'dbus-1', 'services')
)

# gschema
install_data('de.haeckerfelix.Shortwave.gschema.xml', install_dir: join_paths(shortwave_datadir, 'glib-2.0', 'schemas'))
