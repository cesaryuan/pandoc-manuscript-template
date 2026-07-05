Attribute VB_Name = "MTW5"
Attribute VB_Base = "0{FCFB3D2A-A0FA-1068-A738-08002B3371B5}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False
'MTW5 5.0
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTDLLPath.cls 42    10/11/11 2:12p Jimm $
Option Explicit

' Note: * all functions/subs that start with MTCommand_ are "top-level" entry points called from
'         OnAction handlers (e.g. buttons, menu items, etc).
'       * all functions/subs that start with MTCallback_ are are called by
'         Word 2007 as needed to refresh the Ribbon UI
'       * all functions/subs that start with NoDirectCall_
'          MUST NOT BE CALLED except via a RunXXX command


' module name
Private Const module As String = "MTW5"
Private Const moduleCLS As String = "UIWrappersCls"

Public WithEvents App As Application
Attribute App.VB_VarHelpID = -1

Private Sub App_DocumentBeforeSave(ByVal Doc As Document, SaveAsUI As Boolean, Cancel As Boolean)
    If MTMacrosInitialized Then
        RunDocCallback moduleCLS, "NoDirectCall_App_DocumentBeforeSave"
    End If
End Sub

'Runs when a doc is created, opened or activated (W2000 and higher only)
'Checks if doc is marked as containing eqns from other platform
Private Sub App_DocumentOpen(ByVal Doc As Document)
    If MTMacrosInitialized Then
        Set MTW5_aDoc = Doc
        RunDocCallback moduleCLS, "NoDirectCall_App_DocumentOpen"
    End If
End Sub

'Runs when doc is created, opened or activated
'Check if doc's template has old MT macros installed
Private Sub App_DocumentChange()
    If MTMacrosInitialized Then
        RunDocCallback moduleCLS, "NoDirectCall_App_DocumentChange"
    End If
End Sub

Private Sub App_WindowBeforeDoubleClick(ByVal sel As Selection, Cancel As Boolean)
On Error GoTo bye
    If sel.InlineShapes.count = 1 Then
        Dim mtEqn As InlineShape
        Set mtEqn = sel.InlineShapes(1)
        If Strings.left$(mtEqn.OLEFormat.ClassType, 9) = "Equation." And MTIsFullFunctionality(False) <> 1 Then
            Dim result As String
            result = MsgBox(UILib.GetUserString("!3017This copy of MathType is not activated. Convert this MathType equation into an Office equation?"), _
                    vbExclamation + vbYesNoCancel, "MathType")
            If (result = vbYes) Then
                
                Dim mml As String
                Dim mtObj As Object
                
                mtEqn.OLEFormat.DoVerb (2) 'activates the mt ole object
                Set mtObj = mtEqn.OLEFormat.Object
                mml = mtObj.GetMathML
                mtObj.Close
                sel.delete
                MTSetClipboardText (InsertNamespace(mml))
                Selection.Paste
                Cancel = True
            ElseIf (result = vbCancel) Then
                Cancel = True
            End If
        End If
    End If
    Exit Sub
bye:
    WriteLog ("error in double click callback function: " & err.Description)
End Sub

Private Sub App_WindowActivate(ByVal Doc As Document, ByVal Wn As Window)

    If MTMacrosInitialized Then
        RunDocCallback moduleCLS, "NoDirectCall_App_WindowActivate"
    Else

    'the following call is just to catch the exception that is thrown after eXstyles export, when any OleObject is corrupt.
    On Error GoTo Catch:
    Dim bool As Boolean
        bool = Application.ActiveWindow.Selection.Information(wdCapsLock)
Catch:
    'we just silent the error
    End If
End Sub

'Private Sub App_WindowSelectionChange(ByVal sel As Selection)
'    RunDocCallback moduleCLS, "NoDirectCall_App_WindowSelectionChange"
'End Sub

' called when the class is being initialized
Private Sub Class_Initialize()
    On Error GoTo bye
    
    If Not MTMacrosInitialized Then
        Exit Sub
    End If
    
    UIUpdate
    Exit Sub
bye:
        WriteLog "Class_Initialize error"
    ' First, pass along any DSI_ABORT_EXCEPTIONs to be caught by the top-level handler
    If err.Number = DSI_ABORT_EXCEPTION Then
        err.Raise DSI_ABORT_EXCEPTION 'leaves other Err data intact
    Else
        ' handle unexpected error by displaying message, log error, throw DSI private error
        Asserts.AssertFailure module, "Class_Initialize", err.Description
    End If
End Sub
