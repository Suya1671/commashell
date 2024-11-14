# I have no clue what I am doing.
# If you are reading this and do have a clue,
# feel free to make a PR/Issue on fixing the absolute nonsense below.
project('commashell', 'rust',
  version : '0.1',
  default_options : ['warning_level=3'])

i18n = import('i18n')
gnome = import('gnome')

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')

dependency('gtk4', version : '>=4.16')
dependency('libadwaita-1', version : '>=1.6')

base_id = 'in.wobbl.commashell'
application_id = 'in.wobbl.commashell'
resources_path = '/in/wobbl/commashell/'

glib_compile_resources = find_program('glib-compile-resources', version: '>= 2.66', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', version: '>= 2.66', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstreamcli = find_program('appstreamcli', required: false)
cargo = find_program('cargo', required: true)
libspelling = dependency('libspelling-1', required: false)

version = meson.project_version()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
podir = meson.project_source_root() / 'po'
gettext_package = meson.project_name()

if get_option('profile') == 'development'
  profile = 'Devel'
  name_suffix = ' (Devel)'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
  application_id = '@0@.@1@'.format(base_id, profile)
else
  profile = ''
  name_suffix = ''
  version_suffix = ''
  application_id = base_id
endif

subdir('data')
subdir('po')
subdir('src')

gnome.post_install(
  gtk_update_icon_cache: false, # todo: add icons
  glib_compile_schemas: true,
  update_desktop_database: true,
)