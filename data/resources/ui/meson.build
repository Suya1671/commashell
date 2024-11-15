blps = [
    'top.blp',
]

uis = []
foreach blp : blps
    uis += blp.replace('.blp', '.ui')
endforeach

blueprint_compiler = find_program('blueprint-compiler', version: '>= 0.8.0')

blueprints = custom_target(
    'blueprints',
    input: blps,
    output: uis,
    command: [
        blueprint_compiler,
        'batch-compile',
        '@OUTDIR@',
        '@CURRENT_SOURCE_DIR@',
        '@INPUT@',
    ],
)