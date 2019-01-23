subdir('icons')

# desktop file
desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
i18n.merge_file ('desktop-file',
  type: 'desktop',
  input: configure_file(
    input: files('de.haeckerfelix.Shortwave.desktop.in.in'),
    output: 'de.haeckerfelix.Shortwave.desktop.in',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: join_paths (datadir, 'applications')
)

# appdata
appdata_conf = configuration_data()
appdata_conf.set('appid', application_id)
i18n.merge_file ('appdata-file',
  input: configure_file(
    input: files('de.haeckerfelix.Shortwave.appdata.xml.in.in'),
    output: 'de.haeckerfelix.Shortwave.appdata.xml.in',
    configuration: appdata_conf
  ),
  output: '@0@.appdata.xml'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: join_paths (datadir, 'metainfo')
)

# dbus service
service_conf = configuration_data()
service_conf.set('appid', application_id)
service_conf.set('bindir', shortwave_bindir)
configure_file(
  input: 'de.haeckerfelix.Shortwave.service.in',
  output: '@0@.service'.format(application_id),
  configuration: service_conf,
  install_dir: join_paths(datadir,'dbus-1', 'services')
)

# gschema
install_data('de.haeckerfelix.Shortwave.gschema.xml', install_dir: join_paths(get_option('datadir'), 'glib-2.0', 'schemas'))
