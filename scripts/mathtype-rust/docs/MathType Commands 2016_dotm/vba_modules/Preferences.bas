Attribute VB_Name = "Preferences"

'====================================================================
' (c) Copyright 1992-2011 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/Preferences.bas 2     5/06/14 9:54a Jimm $
'====================================================================

' Routines for reading/writing user preferences

Option Explicit

Public Function GetPreference(ByVal hive As Long, ByVal strFolder As String, ByVal strKey As String) As String

#If Win32 Then

    GetPreference = GetRegistryString(hive, strFolder, strKey)

#Else

    Dim stat As Long
    Dim value As String
    Const BUFSIZE As Integer = 1024

    value = Strings.Space(BUFSIZE)

    stat = MTGetPreference(strKey, value, BUFSIZE - 1, "", "")

    value = Strings.Trim(value)

    GetPreference = value

#End If

End Function

Public Sub SetPreference(ByVal hive As Long, ByVal strFolder As String, ByVal strKey As String, ByVal strValue As String)

#If Win32 Then
    SetRegistryString hive, strFolder, strKey, strValue
#Else
    Dim stat As Long
    stat = MTSetPreference(strKey, strValue, "", "")
#End If

End Sub
