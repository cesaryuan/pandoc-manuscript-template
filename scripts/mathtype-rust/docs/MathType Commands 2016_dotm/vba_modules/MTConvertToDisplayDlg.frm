Attribute VB_Name = "MTConvertToDisplayDlg"
Attribute VB_Base = "0{3A110C36-438E-444A-B611-FD8F70642B84}{41FE9610-A306-4A91-AB28-907DC4AFD970}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False



Private Sub UserForm_Initialize()
    Dim fontSize As Long
    Dim fontName As String
    Dim myControl As MSForms.control

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!001111"))
    'Me.height = 4500
    'Me.width = 5295
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 177 '3120
    'Me.width = 240  '4710
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
    MTTeXToggle.gMTConvertToDisplayResult = "cancel"
    Application.Activate 'in case MT had to be started up, grab the focus back
End Sub

Private Sub btnCancel_Click()
    MTTeXToggle.gMTConvertToDisplayResult = "cancel"
    unload Me
End Sub

Private Sub btnConvertToDisplay_Click()
    MTTeXToggle.gMTConvertToDisplayResult = "display"
    unload Me
End Sub

Private Sub btnLeaveInline_Click()
    MTTeXToggle.gMTConvertToDisplayResult = "inline"
    unload Me
End Sub


