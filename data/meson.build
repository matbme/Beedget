desktop_file = i18n.merge_file(
  input: 'com.github.matbme.beedget.desktop.in',
  output: 'com.github.matbme.beedget.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'applications')
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif

appstream_file = i18n.merge_file(
  input: 'com.github.matbme.beedget.appdata.xml.in',
  output: 'com.github.matbme.beedget.appdata.xml',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'appdata')
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test('Validate appstream file', appstream_util,
    args: ['validate', appstream_file]
  )
endif

install_data('com.github.matbme.beedget.gschema.xml',
  install_dir: join_paths(get_option('datadir'), 'glib-2.0/schemas')
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file', compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()]
  )
endif

blueprints = custom_target('blueprints',
  input: files(
    'ui/create-group-dialog.blp',
    'ui/create-transaction-dialog.blp',
    'ui/date-time-picker.blp',
    'ui/group-list-row-content.blp',
    'ui/transaction-row.blp',
    'ui/window.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())
gnome.compile_resources('beedget',
  'beedget.gresource.xml',
  gresource_bundle: true,
  dependencies: blueprints,
  install: true,
  install_dir: pkgdatadir,
  source_dir: ['icons/symbolic/scalable/actions']
)

subdir('icons')
