# distutils/build-windows.nu
#
# Run from the repository root:
#   nu distutils/build-windows.nu

def build-sn8tool [sn8_root: string] {
    let project_root = ($sn8_root | path expand)
    let venv_dir     = $"($project_root)/.venv_sn8tool"
    let build_dir    = $"($project_root)/win"
    let entry_script = "sn8tool.py"
    let cfg_rel_dir  = "sn8"
    let cfg_file     = "sn8f2288.cfg"

    print $"=== [sn8tool] Project root: ($project_root)"

    # Prepare build directory
    if ($build_dir | path exists) == false {
        mkdir $build_dir
    }

    # Ensure virtual environment
    let venv_py = (
        if ($venv_dir | path exists) {
            print $"=== [sn8tool] Using existing venv: ($venv_dir)"
            $"($venv_dir)/Scripts/python.exe"
        } else {
            print $"=== [sn8tool] Creating venv: ($venv_dir)"
            try {
                ^py -3 -m venv $venv_dir
            } catch {
                ^python -m venv $venv_dir
            }
            $"($venv_dir)/Scripts/python.exe"
        }
    )

    # Install / update build dependencies
    print "=== [sn8tool] Installing / updating build dependencies ==="
    ^$venv_py -m pip install --upgrade pip
    ^$venv_py -m pip install --upgrade nuitka libusb1 ply PySide6

    # Build with Nuitka
    print "=== [sn8tool] Building with Nuitka ==="
    let cfg_path = $"($project_root)/config.yaml"

    mut nuitka_args = [
        "-m" "nuitka"
        "--standalone"
        "--enable-plugin=pyside6"
        "--no-deployment-flag=self-execution"
        "--windows-console-mode=disable"
    ]

    if ($cfg_path | path exists) {
        print $"    Using config.yaml: ($cfg_path)"
        $nuitka_args = ($nuitka_args ++ [ $"--user-package-configuration-file=($cfg_path)" ])
    } else {
        print "    config.yaml not found; running Nuitka without user config"
    }

    $nuitka_args = ($nuitka_args ++ [ $"--output-dir=($build_dir)" $entry_script])

    cd $project_root
    ^$venv_py ...$nuitka_args

    # Package sn8tool.dist -> sn8tool and copy cfg
    print "=== [sn8tool] Packaging build results ==="
    let dist_dir = $"($build_dir)/sn8tool.dist"
    let out_dir  = $"($build_dir)/sn8tool"

    if ($out_dir | path exists) {
        print $"    Removing old directory: ($out_dir)"
        rm -rf $out_dir
    }

    if ($dist_dir | path exists) == false {
        error make { msg: $"sn8tool: dist directory not found: ($dist_dir)" }
    }

    mv $dist_dir $out_dir

    let cfg_src     = $"($project_root)/($cfg_rel_dir)/($cfg_file)"
    let cfg_dst_dir = $"($out_dir)/($cfg_rel_dir)"
    let cfg_dst     = $"($cfg_dst_dir)/($cfg_file)"

    if ($cfg_src | path exists) {
        print $"    Copying cfg: ($cfg_src) -> ($cfg_dst)"
        if ($cfg_dst_dir | path exists) == false {
            mkdir $cfg_dst_dir
        }
        cp $cfg_src $cfg_dst
    } else {
        print $"    [WARN] Config not found at: ($cfg_src)"
    }
}

def build-frontend [root: string] {
    let root = ($root | path expand)
    cd $root

    print "=== 1. Compiling TailwindCSS ==="
    ^npm install
    ^npx tailwindcss -i ./input.css -o ./public/tailwind.css --minify

    print "=== 2. Building Dioxus desktop app ==="
    ^dx build --release --platform desktop
}

def assemble-release [root: string, workdir: string, distdir: string, archive_name: string] {
    let root    = ($root | path expand)
    let workdir = ($workdir | path expand)
    let distdir = ($distdir | path expand)

    let app_dir = $"($root)/target/dx/ku1255-firmware-modifier/release/windows/app"
    let exe     = $"($app_dir)/ku1255-firmware-modifier.exe"
    let assets  = $"($app_dir)/assets"

    print "=== 3. Preparing output directories ==="
    if ($workdir | path exists) {
        rm -rf $workdir
    }
    mkdir $workdir

    if ($distdir | path exists) == false {
        mkdir $distdir
    }

    print "=== 4. Copying built binary and assets ==="
    if ($exe | path exists) == false {
        error make { msg: $"Executable not found at: ($exe)" }
    }
    cp $exe $workdir

    if ($assets | path exists) {
        cp -r $assets $"($workdir)/assets"
    }

    print "=== 5. Copying project resources ==="
    let firmware_dir = ($workdir | path join "firmware")
    if not ($firmware_dir | path exists) {
        mkdir $firmware_dir
        touch ($firmware_dir | path join "memo.txt")
    }
    let project_dirs = [ "boards" "examples" "logical_layouts" "settings" "template" ]
    for d in $project_dirs {
        let src = $"($root)/($d)"
        if ($src | path exists) {
            print $"    [+] ($d)"
            cp -r $src $"($workdir)/($d)"
        } else {
            print $"    [-] ($d) (skipped; not found)"
        }
    }

    # Optionally include built sn8tool into the release payload
    let sn8tool_out = $"($root)/sn8files/win/sn8tool"
    if ($sn8tool_out | path exists) {
        print "=== 6. Copying sn8tool binaries ==="
        cp -r $sn8tool_out $"($workdir)/sn8tool"
        cp -r $sn8tool_out $"($root)/sn8tool"
    }

    print "=== 7. Creating ZIP archive ==="
    let archive_path = $"($distdir)/($archive_name)"
    if ($archive_path | path exists) {
        rm $archive_path
    }

    # Use PowerShell's Compress-Archive for portability on Windows
    print $"    Archiving to: ($archive_path)"
    cd $workdir
    ^powershell -Command $"Compress-Archive -Path * -DestinationPath '($archive_path)'"

    if ($archive_path | path exists) == false {
        error make { msg: $"ZIP compression failed; archive not found: ($archive_path)" }
    } else {
        print $"    Archive created successfully: ($archive_path)"
        cd $root
        rm -r $workdir
    }   

    print $"=== Build complete: ($archive_path) ==="
}

def main [] {
    # Assume we are called from the repository root
    let root         = (pwd)
    let workdir      = $"($root)/deploy/windows_working"
    let distdir      = $"($root)/deploy/windows"
    let archive_name = "ku1255-firmware-modifier-windows.zip"
    let sn8_root     = $"($root)/sn8files"

    if ($sn8_root | path exists) == false {
        print $"[WARN] sn8tool directory not found at: ($sn8_root)"
    } else {
        build-sn8tool $sn8_root
    }

    build-frontend $root
    assemble-release $root $workdir $distdir $archive_name
}
