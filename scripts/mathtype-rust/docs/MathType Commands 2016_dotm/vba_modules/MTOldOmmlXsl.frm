Attribute VB_Name = "MTOldOmmlXsl"
Attribute VB_Base = "0{49B21105-B736-4877-BCBB-BD9C24A3501D}{073EDE44-021F-41DC-A27A-1B177BAEE996}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False




'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTOldOmmlXsl.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
Option Explicit

Private Sub btnYes_Click()
    Dim result As Long
    result = MTGetURL(mturlMATHTYPE_OMML2MATHMLXSL, True, "", 0)
    unload Me
End Sub

Private Sub btnNo_Click()
    unload Me
End Sub

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim fontName As String
    Dim fontSize As Long

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!001111"))
    'Me.height = 1365
    'Me.width = 5865
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 94.25   '1365
    'Me.width = 302.75   '5865
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize

    'assign the captions according to the current language
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName
        'myControl.font.size = fontSize
        myControl.caption = MTLib.GetUserString(myControl.caption)
                #If Mac Then
                If TypeOf myControl Is MSForms.Frame Then
                        myControl.BackColor = &H8000000F
                End If
                #End If
    Next

End Sub

