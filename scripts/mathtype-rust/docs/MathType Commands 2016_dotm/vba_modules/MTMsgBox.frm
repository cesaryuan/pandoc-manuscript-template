Attribute VB_Name = "MTMsgBox"
Attribute VB_Base = "0{48895FC1-4AD1-4425-9D64-1CA7B4B90B7F}{1A7163C2-F4E2-4E7A-A245-FAC21B560D98}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False


'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTMsgBox.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
Option Explicit

'Hides buttons as needed, sets title and contents
Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim fontName$
    Dim fontSize As Long

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!001111"))
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize
   '
    If MTLib.gMBStyle = mt_MBYESNO Then
        btnYNC_YES.visible = False
        btnYNC_NO.visible = False
        btnYNC_CANCEL.visible = False
        
        btnYN_YES.default = True
        btnYN_NO.Cancel = True
    ElseIf MTLib.gMBStyle = mt_MBYESNOCANCEL Then
        btnYN_YES.visible = False
        btnYN_NO.visible = False
        
        btnYNC_YES.default = True
        btnYNC_CANCEL.Cancel = True
    End If
    If MTLib.gMBMessage <> "" Then
        lblMessage = MTLib.gMBMessage
    End If
    'init captions
    If MTLib.gMBCaption <> "" Then
        Me.caption = MTLib.gMBCaption
    Else
        Me.caption = MTLib.GetUserString(Me.caption)
    End If
    For Each myControl In Me.Controls
        'myControl.Font.name = fontName$
        'myControl.Font.size = fontSize
        If myControl.name <> "lblMessage" And myControl.visible Then
            myControl.caption = MTLib.GetUserString(myControl.caption)
        End If
                #If Mac Then
                If TypeOf myControl Is MSForms.Frame Then
                        myControl.BackColor = &H8000000F
                End If
                #End If
    Next myControl
End Sub

Private Sub btnYNC_YES_Click()
    MTLib.gMBResult = mt_MBYES
    unload Me
End Sub
Private Sub btnYNC_NO_Click()
    MTLib.gMBResult = mt_MBNO
    unload Me
End Sub
Private Sub btnYNC_CANCEL_Click()
    MTLib.gMBResult = mt_MBCANCEL
    unload Me
End Sub
Private Sub btnYN_YES_Click()
    MTLib.gMBResult = mt_MBYES
    unload Me
End Sub
Private Sub btnYN_NO_Click()
    MTLib.gMBResult = mt_MBNO
    unload Me
End Sub

