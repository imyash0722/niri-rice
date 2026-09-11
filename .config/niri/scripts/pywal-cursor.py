#!/usr/bin/env python3
"""Compatibility wrapper forwarding to matugen-theme.py"""
import os
import sys

script_dir = os.path.dirname(os.path.abspath(__file__))
matugen_theme = os.path.join(script_dir, "matugen-theme.py")
os.execv(sys.executable, [sys.executable, matugen_theme] + sys.argv[1:])
