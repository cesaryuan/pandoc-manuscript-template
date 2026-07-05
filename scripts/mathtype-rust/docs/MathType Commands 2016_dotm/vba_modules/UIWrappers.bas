Attribute VB_Name = "UIWrappers"
'UIWrappers: 5.01
'====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved
'$Header: /MathType/Windows/WordMacros/UIWrappers.bas 28    7/12/10 1:37p Jimm $
'====================================================================
Option Explicit

' Note: * all functions/subs that start with MTCommand_ are "top-level" entry points called from
'         OnAction handlers (e.g. buttons, menu items, etc).
'       * all functions/subs that start with MTCallback_ are are called by
'         Word 2007 as needed to refresh the Ribbon UI
'       * all functions/subs that start with NoDirectCall_
'          MUST NOT BE CALLED except via a RunXXX command

Private Const module As String = "UIWrappers"
Private Const moduleCLS As String = "UIWrappersCls"

' *** Begin MTW5 Globals ***
Public MTW5_aDoc As Document
' *** End MTW5 Globals ***

' this is called when the user selects File->Print Preview
' This should start with MTCommand_ but it can not since Word expects this name
Public Sub FilePrintPreview()
    ' We have to use RunDocCallback here, since concurrent execution is
    ' required for print preview.  See MT-1119
    RunDocCallback moduleCLS, "NoDirectCall_FilePrintPreview"
End Sub

'Called by Print button that has no UI
' This should start with MTCommand_ but it can not since Word expects this name
Public Sub FilePrintDefault()
    ' We have to use RunDocCallback here, since concurrent execution is
    ' required for print preview.  See MT-1119
    RunDocCallback moduleCLS, "NoDirectCall_FilePrintDefault"
End Sub

'Called by Print command, shows dialog
' This should start with MTCommand_ but it can not since Word expects this name
Public Sub FilePrint()
    RunMTDLLCommand moduleCLS, "NoDirectCall_FilePrint"
End Sub

'Starting with Word 2002, MTEF comment records are stripped out of WMF files
' by Word when they are placed on the clipboard.  Thus, in recent versions of
' Word, overriding the native EditPictureEdit command does no good in terms
' of being ablt to edit MT picture equations as equations. Moreover, in Word 2007
' calling the Activate method on an inline shape no longer invokes the default
' picture editor in Word, so it becomes quite difficult to hook the EditPictureEdit
' command without screwing up the native picture editing capabilities in Word.
' Thus, we remove this override in MT6.0b.  See MT-1131.
'Public Sub EditPictureEdit()
'    RunMTDLLCommand "", kMTCommands & ".MTLib.MTEditPicture", False
'End Sub


'Inserts an equation at the insertion point
'InsertEquation is called when the EE (square root of alpha) button is clicked
' This should start with MTCommand_ but it can not since Word expects this name
Public Sub InsertEquation()
    RunMTDLLCommand moduleCLS, "NoDirectCall_InsertEquation"
End Sub

'Shows/hides all nonprinting characters, and our Equation Breaks (avoids dirtying the doc)
' This should start with MTCommand_ but it can not since Word expects this name
Public Sub ShowAll()
    RunMTDLLCommand moduleCLS, "NoDirectCall_ShowAll"
End Sub

'This macro replaces one shipped with MathType1.x.
'Run when user clicks on an old paragraph (display) MathType equation.
' This should start with MTCommand_ but it can not since older MT versions expect this name
Public Sub EditTextEqn()
    RunMTDLLCommand moduleCLS, "NoDirectCall_EditTextEqn"
End Sub

'This macro replaces one shipped with MathType 1.x.
'Run when user clicks on an old inline (text) macrobutton MathType equation.
' This should start with MTCommand_ but it can not since older MT versions expect this name
Public Sub EditDispEqn()
    RunMTDLLCommand moduleCLS, "NoDirectCall_EditDispEqn"
End Sub
