"""Exercise the built C library from Python, across all four exported widths.

Usage: python3 experiments/issue-22/ffi-smoke.py target/release/libdoublets_ffi.so
On macOS use libdoublets_ffi.dylib; on Windows use doublets_ffi.dll.
"""
import ctypes as c
from pathlib import Path
import sys
import tempfile

library = c.CDLL(str(Path(sys.argv[1]).resolve()))
for prefix, integer in [('Byte', c.c_uint8), ('UInt16', c.c_uint16),
                        ('UInt32', c.c_uint32), ('UInt64', c.c_uint64)]:
    class Link(c.Structure):
        _fields_ = [('index', integer), ('source', integer), ('target', integer)]

    class Range(c.Structure):
        _fields_ = [('start', integer), ('end', integer)]

    class Constants(c.Structure):
        _fields_ = [(name, integer) for name in (
            'index_part', 'source_part', 'target_part', 'null', 'continue_',
            'break_', 'skip', 'any', 'itself', 'error')]
        _fields_ += [('internal_range', Range), ('external_range', Range),
                     ('has_external_range', c.c_bool)]

    callback = c.CFUNCTYPE(integer, Link, Link)
    new = getattr(library, prefix + 'Links_New')
    new.argtypes, new.restype = [c.c_char_p], c.c_void_p
    drop = getattr(library, prefix + 'Links_Drop')
    drop.argtypes, drop.restype = [c.c_void_p], None
    get_constants = getattr(library, prefix + 'Links_GetConstants')
    get_constants.argtypes, get_constants.restype = [c.c_void_p], Constants
    create = getattr(library, prefix + 'Links_SmartCreate')
    create.argtypes, create.restype = [c.c_void_p], integer
    count = getattr(library, prefix + 'Links_Count')
    count.argtypes, count.restype = [c.c_void_p, c.POINTER(integer), c.c_size_t], integer
    delete = getattr(library, prefix + 'Links_Delete')
    delete.argtypes, delete.restype = [c.c_void_p, c.POINTER(integer), c.c_size_t, callback], integer
    continue_value = 0
    events = []

    @callback
    def deleted(before, after):
        events.append((before.index, after.index))
        return continue_value

    with tempfile.TemporaryDirectory() as directory:
        path = str(Path(directory) / 'links').encode()
        handle = new(path)
        assert handle, prefix
        try:
            constants = get_constants(handle)
            continue_value = constants.continue_
            assert continue_value != constants.break_
            assert create(handle) == 1
            assert count(handle, None, 0) == 1
            result = delete(handle, (integer * 1)(1), 1, deleted)
            assert result == continue_value, (prefix, result, continue_value)
            assert events == [(1, 0)], (prefix, events)
            assert count(handle, None, 0) == 0
        finally:
            drop(handle)
        handle = new(path)
        assert handle, prefix
        try:
            assert count(handle, None, 0) == 0
        finally:
            drop(handle)
    print(prefix + ': C ABI, callbacks, disposal, and reopening passed')
