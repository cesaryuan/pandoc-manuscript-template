Attribute VB_Name = "Asserts"
'Asserts
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/Asserts.bas 11    10/17/11 11:13a Jimm $
'=====================================================================

' This is our private exception which starts at the lowest recommended user defined error number
Public Const DSI_ABORT_EXCEPTION As Long = vbObjectError + 512 + 1

' Used by CallbackGuard and CommandGuard classes... do not edit this value from
' outside of these classes
Public macroExecuting As Boolean
Public gVerboseLogging As Boolean
Private Const doAsserts = False

#If False Then
Public Sub WriteLog(msg As String)

    If gVerboseLogging = False Then
        Exit Sub
    End If

    On Error GoTo bye
    Dim folder As String
    Dim fileNum As Integer
    fileNum = 0
    Dim logFileName As String
    logFileName = "MT_VBA_Asserts.log"
    Dim logFilePath As String
    
    #If Win32 Then
        Dim fso As Object
        Set fso = CreateObject("Scripting.FileSystemObject")
    folder = fso.GetSpecialFolder(2)
        folder = folder & "\"
    #Else
        'Application.PathSeparator is undefined in Office 2011
        #If Word Then
        folder = Application.path & ":"
        #Else
        folder = UILib.GetMathTypeDir & ":"
        #End If
    #End If

    logFilePath = folder & logFileName
    fileNum = FileSystem.FreeFile
    If fileNum <> 0 Then
        Open logFilePath For Append Access Write Shared As #fileNum
        Print #fileNum, Now & ": "; msg
    End If
bye:
    If fileNum <> 0 Then
        Close #fileNum
    End If
End Sub
#End If

Public Sub AssertFailure(module As String, method As String, msg As String)
    WriteLog msg 'Log the error to a file in the user temp directory
    If doAsserts = True Then
         'Dim dlg As AssertDlg
         'Set dlg = New AssertDlg
         '#If Mac And PP Then
         'dlg.Message.Caption = ReplaceSubstring(dlg.Message.Caption, "x", module & "." & method & " " & msg)
         '#Else
         'dlg.Message.Caption = replace(dlg.Message.Caption, "x", module & "." & method & " " & msg)
         '#End If
         '
         'dlg.Show
         '
         'If dlg.tag = "Ignore" Then
         '    Exit Sub
         'ElseIf dlg.tag = "Break" Then
         '    Stop
         '    err.Raise DSI_ABORT_EXCEPTION
         'End If
     Else
        MsgBox "MathType has detected an error in" & module & "." & method & ": " & msg & ". Please save your " & _
               "document and report this error to Design Science Technical Support."
        err.Raise DSI_ABORT_EXCEPTION
     End If
End Sub

Public Sub Assert(b As Boolean, module As String, method As String, msg As String)
    If b = False Then
        AssertFailure module, method, msg
    End If
End Sub

Public Sub LogAndAlert(module As String, method As String, msg As String)
    On Error GoTo done
    'Increment a version check variable counter.
    MTIncrementStatisticBy "OffAst", 1
    AssertFailure module, method, msg
done:
End Sub
