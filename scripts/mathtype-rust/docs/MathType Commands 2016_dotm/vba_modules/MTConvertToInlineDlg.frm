Attribute VB_Name = "MTConvertToInlineDlg"
Attribute VB_Base = "0{488FF4BC-9D08-4849-BBAF-7FFFBC2B5D2C}{8E8B9984-93D3-4E5C-B5F6-58C749C660E9}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False




'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTConvertToInlineDlg.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
Option Explicit

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim settings
    Dim fontName$
    Dim fontSize As Long

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!001111"))
    'Me.height = 2400
    'Me.width = 5880
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 133.5   '2250
    'Me.width = 203.25   '3975
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize

    'assign the captions according to the current language
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName$
        'myControl.font.size = fontSize
        myControl.caption = MTLib.GetUserString(myControl.caption)
        #If Mac Then
        If TypeOf myControl Is MSForms.Frame Then
            myControl.BackColor = &H8000000F
        End If
        #End If
    Next myControl
    Application.Activate 'in case MT had to be started up, grab the focus back
End Sub
Private Sub btnConvert_Click()
    MTTeXToggle.gMTConvertToInlineResult = "inline"
    unload Me
End Sub
Private Sub btnCancel_Click()
    MTTeXToggle.gMTConvertToInlineResult = "cancel"
    unload Me
End Sub

