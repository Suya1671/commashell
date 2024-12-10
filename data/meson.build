# Appdata
appdata_conf = configuration_data()
appdata_conf.set('app-id', application_id)
appdata_conf.set('gettext-package', gettext_package)
appdata_file = configure_file(
  input: '@0@.metainfo.xml'.format(base_id),
  output: '@0@.metainfo.xml'.format(application_id),
  configuration: appdata_conf,
  install: true,
  install_dir: datadir,
)

# Validate Appdata
if appstreamcli.found()
  test(
    'validate-appdata',
    appstreamcli,
    args: ['validate', '--no-net', '--explain', appdata_file],
  )
endif

# subdir('icons')
subdir('resources')

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = configure_file(
  input: '@0@.desktop.in.in'.format(base_id),
  output: '@0@.desktop'.format(application_id),
  configuration: desktop_conf,
  install: true,
  install_dir: datadir / 'applications',
)

# Validate Desktop file
if desktop_file_validate.found()
  test(
    'validate-desktop',
    desktop_file_validate,
    args: [desktop_file.full_path()],
  )
endif

# GSchema
gschema_conf = configuration_data()
gschema_conf.set('app-id', application_id)
gschema_file = configure_file(
  input: 'in.wobbl.commashell.gschema.in.xml',
  output: 'in.wobbl.commashell.gschema.xml',
  configuration: gschema_conf,
  install: true,
  install_dir: datadir / 'glib-2.0' / 'schemas',
)

install_data(
  gschema_file,
  install_dir: datadir / 'glib-2.0' / 'schemas',
)
gnome.compile_schemas()

# Validata GSchema
if glib_compile_schemas.found()
  test(
    'validate-gschema',
    glib_compile_schemas,
    args: ['--strict', '--dry-run', meson.current_build_dir()],
  )
endif