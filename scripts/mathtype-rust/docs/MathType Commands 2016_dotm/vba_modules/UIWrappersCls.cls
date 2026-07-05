Attribute VB_Name = "UIWrappersCls"
Attribute VB_Base = "0{FCFB3D2A-A0FA-1068-A738-08002B3371B5}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False
Private m_lError As Long
Private m_sError As String

'=====================================================================
' (c) Copyright 2007-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/UIWrappersCls.cls 25    5/06/14 9:54a Jimm $
'=====================================================================

' Support code for updating the cursor position
Private Const mttstyle_EB_NUMBERED_EQUATION As String = "MTEBNumberedEquation"

' Expose the error description
' This error number is required because the CallByName
' function does not return the correct error
Public Property Get ErrorDescription() As String
    ErrorDescription = m_sError
End Property

' Expose the error number
' This error number is required because the CallByName
' function does not return the correct error
Public Property Get ErrorNumber() As Long
    ErrorNumber = m_lError
End Property


'************* UIWrappers Module code *****************

Public Sub NoDirectCall_EditPaste()

    RunDispatch "", kMTCommands & ".MTLib.MTEditPaste", False

End Sub

Public Sub NoDirectCall_App_DocumentChange()
    ' It's screwy to look for MT3 macros everytime we change to
    ' a document. Not only will the call be redundant 99% of the time,
    ' it is not clear that MT3 macros can be in documents from any of
    ' the versions of Word we still support. Further, our AutoExec code
    ' string MT3 macros from Normal.dot.  This whole handler should be
    ' removed in future versions of MT.
    Dim aTemplate As Template
    On Error GoTo Error
    'put in variable, can't pass objects directly
    Set aTemplate = ActiveDocument.AttachedTemplate
    UILib.RemoveMT3Macros aTemplate
        Exit Sub
Error:
        WriteLog "NoDirectCall_App_DocumentChange error"
End Sub

Public Sub NoDirectCall_FilePrintPreview()
    Dim Doc As Document
    Dim styleVisible As Boolean

    Set Doc = ActiveDocument
    If Not Doc Is Nothing Then
        If PrintPreview Then
            PrintPreview = False
        Else
            styleVisible = IsSectionStyleVisible()
            If styleVisible Then HideSectionStyle
            Doc.PrintPreview
            ' if the style is visible, we need to loop until the print
            ' preview "mode" ends. We need to make sure that DoEvents
            ' gets called so the user can do other things in the UI
            If styleVisible Then
                Do Until PrintPreview = False
                    DoEvents
                Loop
                ' when the loop ends, show the style
                ShowSectionStyle
            End If
        End If
    End If
End Sub

Public Sub NoDirectCall_FilePrintDefault()
    PrintDoc False
End Sub


Public Sub NoDirectCall_FilePrint()
    PrintDoc True
End Sub

Public Sub NoDirectCall_InsertEquation()
    RunDispatch "", kMTCommands & ".MTInsertEquation.InsertInlineEquation", False
End Sub

Public Sub NoDirectCall_ShowAll()
    Dim saved As Boolean
    saved = ActiveDocument.saved
    If ActiveWindow.ActivePane.View.ShowAll Then
        HideSectionStyle
    Else
        ShowSectionStyle
    End If
    ActiveWindow.ActivePane.View.ShowAll = Not ActiveWindow.ActivePane.View.ShowAll
    ActiveDocument.saved = saved
End Sub

Public Sub NoDirectCall_EditTextEqn()
    Dim stat As Long
    stat = MsgBox(UILib.GetUserString("!2500This is a MathType 1.x (non-OLE) equation. The Convert Equations... command can be used to convert it to a MathType 4 equation. Do you want to do this now?"), _
        vbQuestion + vbYesNo, UILib.GetUserString2("1620", "3220", "MathType Macro Message"))

    If stat = vbYes Then UILib.MTCommand_ConvertEqns
End Sub

Public Sub NoDirectCall_EditDispEqn()
    Dim stat As Long
    stat = MsgBox(UILib.GetUserString("!2500This is a MathType 1.x (non-OLE) equation. The Convert Equations... command can be used to convert it to a MathType 4 equation. Do you want to do this now?"), _
        vbQuestion + vbYesNo, UILib.GetUserString2("1620", "3220", "MathType Macro Message"))

    If stat = vbYes Then UILib.MTCommand_ConvertEqns
End Sub

'Support Functions
'Returns True if style exists in the active document.
Private Function styleExists(style As String) As Boolean
    Dim aStyle As style
    styleExists = False
    On Error GoTo abort
    Set aStyle = ActiveDocument.Styles(style)
    'if we get here the style exists
    styleExists = True
abort:
End Function

'If MTEquationSection style exists, make it not hidden
Private Sub ShowSectionStyle()
    If styleExists(mtstyle_EQUATION_SECTION) Then
        ActiveDocument.Styles(mtstyle_EQUATION_SECTION).font.Hidden = False
    End If
End Sub

'If MTEquationSection style exists, make it hidden
Public Sub HideSectionStyle()
    If styleExists(mtstyle_EQUATION_SECTION) Then
        ActiveDocument.Styles(mtstyle_EQUATION_SECTION).font.Hidden = True
    End If
End Sub

'Returns True if MTEquationSection style is visible
Public Function IsSectionStyleVisible() As Boolean
    IsSectionStyleVisible = False
    If styleExists(mtstyle_EQUATION_SECTION) Then
        IsSectionStyleVisible = _
            Not (ActiveDocument.Styles(mtstyle_EQUATION_SECTION).font.Hidden)
    End If
End Function

'Hides our MTEquationSection style if necessary before printing
Private Sub PrintDoc(ui As Boolean)
    On Error GoTo abort
    Dim styleVisible As Boolean
    styleVisible = IsSectionStyleVisible()
    If styleVisible Then HideSectionStyle
    With Dialogs(wdDialogFilePrint)
        If ui Then
            .Show
        Else
            .Execute
        End If
    End With
    If styleVisible Then ShowSectionStyle
abort:
End Sub

'************* MTW5 class module code *****************

Public Sub NoDirectCall_App_DocumentBeforeSave()
    ' delay 3 seconds and update the UI
    If Not MTApp.App Is Nothing Then
        MTApp.App.OnTime Now + TimeValue("00:00:03"), "UIUpdate"
    End If
End Sub

Public Sub NoDirectCall_App_DocumentOpen()
    CheckEqnsFromOtherPlatform
    InitDefaultEqnNumSide
    Invalidate
End Sub

Private Sub CheckEqnsFromOtherPlatform()
    On Error GoTo MacError

    If (MTW5_aDoc Is Nothing) Then
        Exit Sub
    End If

    If GetPlatform() = kPlatformMac Then
        If MTW5_aDoc.CustomDocumentProperties(mtprop_HAS_WIN_EQNS) = True Then
            DeleteDocProperty MTW5_aDoc, mtprop_HAS_WIN_EQNS
            AskRunConvertEquations kPlatformWin
        End If
    End If

WinTest:
    On Error GoTo WinError
    If GetPlatform() = kPlatformWin Then
        If MTW5_aDoc.CustomDocumentProperties.item(mtprop_HAS_MAC_EQNS).value = True Then
            DeleteDocProperty MTW5_aDoc, mtprop_HAS_MAC_EQNS
            AskRunConvertEquations kPlatformMac
        End If
    End If
    Exit Sub

MacError:
    Resume WinTest:

WinError:
End Sub

'Prompt to run Convert Equations
Private Sub AskRunConvertEquations(platformID As Long)
    Dim stat As Long
    Dim msg As String

    msg = UILib.GetUserString2("0600", "0600", "This document contains equations created on a Macintosh computer. To view the equations correctly you should run the Convert Equations command. Would you like to run this command now?")

    stat = MsgBox(msg, vbQuestion + vbYesNo, UILib.GetUserString("!1806MathType Commands for Word"))
    If stat = vbYes Then
        UILib.MTCommand_ConvertEqns
    End If
End Sub
'Deletes document property, OK to call if it doesn't exist
Public Sub DeleteDocProperty(Doc As Document, prop As String)
    On Error GoTo Error
    Doc.CustomDocumentProperties(prop).delete
Error:
End Sub

'Similar to MTLib.GetPlatform, returns kPlatformWin or kPlatformMac
Private Function GetPlatform() As Long
    If InStr(1, System.OperatingSystem, "Macintosh", vbTextCompare) > 0 Then
        GetPlatform = kPlatformMac
    Else
        GetPlatform = kPlatformWin
    End If
End Function

Public Sub NoDirectCall_App_WindowActivate()
    InitDefaultEqnNumSide
    Invalidate
    UIUpdate
End Sub

Public Sub NoDirectCall_App_WindowSelectionChange()
    ' special handling for hitting return when on the right side of a EB equation
    checkForEBEquationAfterReturn
    UIUpdate
End Sub

' special handling for hitting return when on the right side of a EB equation
Private Sub checkForEBEquationAfterReturn()
#If Win32 Then
    Application.ScreenUpdating = False
    Dim resetSelection As Boolean
    resetSelection = False
    On Error Resume Next
    ' this only applies to Office 2007
    If Val(Application.version) >= kWord2007 Then
        ' use Win32 API to obtain the state of the return key
        ' the high order bit is set to 1 if the return key is down
        ' which looks like a negative integer
        Dim result As Integer
        result = GetKeyState(KeyCodeConstants.vbKeyReturn)
        ' keep the selection in case we need to revert back to it
        Dim savedRng As Range
        Set savedRng = Selection.Range.Duplicate

        ' the following variables are used to achieve late binding
        ' (i.e. for compiling on pre-Office 2007)
        Dim tbl
        Dim sel
        Set sel = Selection
        With sel
            Set tbl = .Tables(1)

            ' EB equations we have inserted are suspposed to have a table around them
            If IsEmpty(tbl) Then
                ' we did not change the selection, so just bail
                Application.ScreenUpdating = True
                Exit Sub
            End If

            ' if the return key is pressed, and we are in a EB equation table,
            ' but we are not inside a EB equation
            If result < 0 And _
                .Information(wdWithInTable) And _
                tbl.style.NameLocal = mttstyle_EB_NUMBERED_EQUATION And _
                sel.OMaths.count = 0 Then

                ' backup one character and see if we are in a EB equation
                .moveLeft

                If sel.OMaths.count > 0 Then
                    ' since we are in a EB equation, move one more to the left,
                    ' which should put the cursor just after the equation,
                    ' but before the return character, and then delete
                    .moveLeft
                    .delete
                    ' now jump out of the table and enter a new paragraph,
                    ' which is what the user really wanted to do
                    While .Information(wdWithInTable)
                        If .MoveRight = 0 Then Exit Sub
                    Wend
                    .TypeParagraph
                    .moveLeft
                Else
                    resetSelection = True
                End If
            End If
        End With
    End If
done:
    If resetSelection Then
        ' we must not be in a EB equation, so reset the selection
        savedRng.Select
    End If
    Application.ScreenUpdating = True
#End If
End Sub

