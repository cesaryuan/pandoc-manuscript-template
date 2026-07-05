Attribute VB_Name = "AutoExec"
'AutoExec 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/AutoExec.bas 56    10/17/11 11:13a Jimm $
'=====================================================================
Option Explicit

Public MTApp As New MTW5

Private Const module As String = "AutoExec"
Private Const moduleCLS As String = "AutoExecCls"

Public MTMacrosInitialized As Boolean ' false when initialized

' This should start with MTCommand_ but it can not since Word expects this name
Public Sub PrivateMain()
    If MTMacrosInitialized Then
        Exit Sub
    End If
    macroExecuting = False
    RunMTDLLCommand moduleCLS, "NoDirectCall_Main"
    MTMacrosInitialized = True
End Sub

Public Sub Main()
    WriteLog "Storing Application in MTApp"
    SetDLLPath
        FixMathPageWllAvailability
    Set MTApp.App = Application
End Sub

Private Function FixMathPageWllAvailability()
On Error GoTo CopyWll
    MTAPIGetNestingLevel
    Exit Function
CopyWll:
    If err.Number = "53" Then
        FileCopy MTLib.GetMathTypeDir & Application.PathSeparator & "MathPage" & Application.PathSeparator & Trim(str(kBits)) & Application.PathSeparator & "MathPage.wll", Application.StartupPath & Application.PathSeparator & "MathPage.wll"
        MsgBox "Please restart Word to load MathType addin properly", vbOKOnly, "MathType"
    End If
    
End Function
