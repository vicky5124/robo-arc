#! /usr/bin/python3
import sys
from googletrans import Translator

translator = Translator()
args = sys.argv[1:]

if len(args) == 2:
    t = translator.rust_translate(args[0], args[1])
else:
    t = translator.rust_translate(args[0], args[1], args[2])
print(t)
