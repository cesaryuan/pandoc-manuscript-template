Attribute VB_Name = "Registry"
'=====================================================================
' (c) Copyright 1992-2011 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/Registry.bas 7     10/11/11 2:12p Jimm $
'=====================================================================
'Registry - a collection of registry functions
'Registry routines that are bitness independent

Option Explicit

'Unicode null-terminated string.
Public Const REG_SZ = 1
Public Const ERROR_SUCCESS = 0&

Public Const STANDARD_RIGHTS_ALL = &H1F0000
Public Const READ_CONTROL = &H20000
Public Const STANDARD_RIGHTS_READ = (READ_CONTROL)
Public Const STANDARD_RIGHTS_WRITE = (READ_CONTROL)
Public Const SYNCHRONIZE = &H100000
Public Const KEY_CREATE_LINK = &H20
Public Const KEY_CREATE_SUB_KEY = &H4
Public Const KEY_ENUMERATE_SUB_KEYS = &H8
Public Const KEY_NOTIFY = &H10
Public Const KEY_QUERY_VALUE = &H1
Public Const REG_OPTION_NON_VOLATILE = &H0
Public Const KEY_SET_VALUE = &H2
Public Const KEY_ALL_ACCESS = ((STANDARD_RIGHTS_ALL Or KEY_QUERY_VALUE Or KEY_SET_VALUE Or KEY_CREATE_SUB_KEY Or KEY_ENUMERATE_SUB_KEYS Or KEY_NOTIFY Or KEY_CREATE_LINK) And (Not SYNCHRONIZE))
