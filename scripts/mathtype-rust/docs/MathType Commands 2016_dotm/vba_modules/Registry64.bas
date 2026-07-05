Attribute VB_Name = "Registry64"
'=====================================================================
' (c) Copyright 1992-2011 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/64-bits/Registry64.bas 1     10/11/11 2:07p Jimm $
'=====================================================================
'Registry - a collection of registry functions
'64-bit Registry routines

Option Explicit

Public Const KEY_WOW64_32 = &H200
'Public Const KEY_WOW64_64 = &H100
Public Const KEY_READ = ((STANDARD_RIGHTS_READ Or KEY_QUERY_VALUE Or KEY_ENUMERATE_SUB_KEYS Or KEY_NOTIFY) And (Not SYNCHRONIZE))
Public Const KEY_WRITE = ((STANDARD_RIGHTS_WRITE Or KEY_SET_VALUE Or KEY_CREATE_SUB_KEY) And (Not SYNCHRONIZE))


'KEY_READ                   131609  0x20219
'STANDARD_RIGHTS_READ       131072  0x20000
'KEY_QUERY_VALUE                 1  0x00001
'KEY_ENUMERATE_SUB_KEYS          8  0x00008
'KEY_NOTIFY                     16  0x00010
'KEY_WOW64_32                  512  0x00200
'total                              0x20219

'=====================================================================
'                   OpenKey
'This function returns a handle to an existing key, or zero if the
'key does not exist or if the function fails.
'---------------------------------------------------------------------
Public Function OpenKey(ByVal hive As Long, ByVal strKey As String) As LongPtr
    Dim lngReturn As Long
    Dim lngKey As LongPtr

    lngReturn = RegOpenKeyEx(hive, strKey, 0&, KEY_READ Or KEY_WOW64_32, lngKey)

    If lngReturn <> ERROR_SUCCESS Then
        lngKey = 0
    End If

    OpenKey = lngKey
End Function

'=====================================================================
'                       GetRegistryString
'Retrieves a string value from a specified key.
'Use HKEY_CURRENT_USER etc. for hive
'Returns an empty string if it fails.
'---------------------------------------------------------------------
Public Function GetRegistryString(ByVal hive As Long, ByVal strKey As String, ByVal strValueName As String) As String

    Dim strBuffer As String, strMsg As String
    Dim lngRetVal As Long
    Dim lngValueType As Long
    Dim lngValueLength As Long
    Dim lngKey As LongPtr

    'Open the existing key.
    lngKey = OpenKey(hive, strKey)
    If lngKey <> 0 Then

        ' get the required buffer size
        lngRetVal = RegQueryValueExNULL(lngKey, strValueName, 0&, lngValueType, 0&, lngValueLength)

        ' create a byte buffer. Using a string buffer results in a failed call to RegQueryValueExString
        ' and crashes PowerPoint
        Dim strBufferUnicode() As Byte
        ReDim strBufferUnicode(lngValueLength + 2)

        ' get the regkey
        lngRetVal = RegQueryValueExString(lngKey, strValueName, 0&, lngValueType, strBufferUnicode(0), lngValueLength)

        If lngRetVal = ERROR_SUCCESS Then
            If lngValueType = REG_SZ Then
                ' convert from byte array to string
                strBuffer = left(StrConv(strBufferUnicode, vbUnicode), lngValueLength - 1)
                strBuffer = Trim(strBuffer)
            Else
                strBuffer = ""
            End If
        Else
            strBuffer = ""
        End If

        'Call the API function to close the key (ignore errors)
       lngRetVal = RegCloseKey(lngKey)
    End If

    GetRegistryString = strBuffer
End Function

' Sets the indicated registry value, creating the key if necessary
Public Sub SetRegistryString(ByVal hive As Long, ByVal strKeyName As String, ByVal strValueName As String, ByVal vValueSetting As Variant)

    Dim lRetVal As Long  'result of the SetValueEx function
    Dim hKey As LongPtr     'handle of open key
    Dim lngDisp As Long  'whether an existing key was opened or a new one created
    Dim sam As Long

    If hive = HKEY_LOCAL_MACHINE Then
        sam = KEY_WRITE Or KEY_WOW64_32
    Else
        sam = KEY_WRITE
    End If

    'open the specified key
    lRetVal = RegCreateKeyEx(hive, strKeyName, 0, 0, REG_OPTION_NON_VOLATILE, sam, 0, hKey, lngDisp&)

    If lRetVal <> ERROR_SUCCESS Then
        hKey = 0
    Else
        Dim strValueSetting As String
        strValueSetting = vValueSetting & Chr$(0)
        RegSetValueExString hKey, strValueName, 0&, REG_SZ, strValueSetting, Len(strValueSetting)
        RegCloseKey (hKey)
    End If

End Sub
