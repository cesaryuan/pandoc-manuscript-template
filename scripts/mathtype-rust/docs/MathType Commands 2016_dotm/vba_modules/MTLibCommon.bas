Attribute VB_Name = "MTLibCommon"

'=====================================================================
' (c) Copyright 1992-2011 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTLibCommon.bas 6     5/06/14 9:54a Jimm $
'=====================================================================

'
' Routines common to Word and PowerPoint
'

Option Explicit

Private gDLLNotFoundErrorShown As Boolean

Public Sub SetDLLPath()
#If Win32 Then
    Dim mtDir As String
    Dim mathPageDir As String
    Dim buf As String
    Dim result As Long
    Dim pathSep As String

    #If Word Then
    pathSep = Application.PathSeparator
    #Else
    pathSep = "\"
    #End If

    mathPageDir = MTLib.GetMathTypeDir & pathSep & "MathPage" & pathSep & Trim(str(kBits))

    If (FileExists(mathPageDir) = False) Then
        mathPageDir = MTLib.GetMathTypeDir & pathSep & "MathPage"
    End If

    ' set the buffer to the max length of the path
    buf = Space(32767)

    ' get the path environment variable
    result = GetEnvironmentVariable("Path", buf, 32767)

    If result <> 0 Then

        ' truncate the path to the proper length of the path
        buf = left(buf, result)

        ' if we have already added the mathpage dir to the path then exit
        If InStr(1, buf, mathPageDir, vbTextCompare) Then
            Exit Sub
        End If

        ' append the mathpage dir to the path
        buf = buf & ";" & mathPageDir
        SetEnvironmentVariable "Path", buf

    End If
#End If
End Sub

Public Function FileExists(fileName As String) As Boolean

    FileExists = False
  
    On Error GoTo done
    If Not dir(fileName, vbDirectory) = vbNullString Then
        FileExists = True
    End If
    
done:
    On Error GoTo 0

End Function

Public Sub ShowDLLNotFoundError()
    If gDLLNotFoundErrorShown = False Then
        MsgBox MTLib.GetUserString("!1697The MathType DLL cannot be found. Please reinstall MathType."), vbCritical, MTLib.GetUserString("!1609MathType Commands for Microsoft Word Error")
        gDLLNotFoundErrorShown = True
    End If
End Sub
