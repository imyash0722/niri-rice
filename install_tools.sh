#!/bin/bash

echo "Starting installation of tools for your coursework..."

# 1. Update system and install available packages via paru
echo "Installing packages via paru (this will prompt for your sudo password)..."

paru -S --needed \
    iverilog \
    gtkwave \
    gcc \
    gdb \
    valgrind \
    make \
    python \
    python-numpy \
    python-scipy \
    python-matplotlib \
    jupyter-notebook \
    octave \
    qemu-full \
    git \
    wireshark-qt \
    riscv64-elf-gcc \
    logisim-evolution-bin \
    vscodium \
    python-cs50 \
    mininet \
    freecad \
    krita

# 2. Build gem5 from source
echo "Setting up gem5..."
if [ ! -d "$HOME/gem5" ]; then
    echo "Cloning gem5 repository..."
    # Install gem5 dependencies
    sudo pacman -S --needed scons m4 protobuf hdf5 python-pydot pkgconf
    git clone https://gem5.googlesource.com/public/gem5 "$HOME/gem5"
    
    read -p "Do you want to compile gem5 now? (Warning: takes a long time, 20-30+ mins) [y/N]: " compile_gem5
    if [[ "$compile_gem5" =~ ^[Yy]$ ]]; then
        echo "Compiling gem5..."
        cd "$HOME/gem5" && scons build/X86/gem5.opt -j$(nproc)
    else
        echo "gem5 cloned to ~/gem5. To build it later, run:"
        echo "cd ~/gem5 && scons build/X86/gem5.opt -j\$(nproc)"
    fi
else
    echo "gem5 directory already exists at ~/gem5"
fi

echo ""
echo "Installation complete!"
echo "Note: QuestaSim and MATLAB require university licenses and are not installed."
