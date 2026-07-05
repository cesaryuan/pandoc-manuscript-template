Attribute VB_Name = "MTProgress"
Attribute VB_Base = "0{FEE1866B-2811-4FE2-B5C3-BA1EF012B914}{81519242-0A48-42AA-B57D-E76956604595}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False


Option Explicit
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTProgress.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
'Use activation message to run MathPage
Private Sub UserForm_Activate()
    Me.Repaint
    If gProgressInfo.procID = kPROGRESS_MATHPAGEPROC Then
        Set gProgressInfo.form = Me
        MathPage.ProcessDocument
    End If
    unload Me
    DoEvents
End Sub

Private Sub UserForm_Initialize()
    Dim offset As Long
    Dim item As MSForms.control
    Dim i As Long

    #If Mac Then
    Me.BackColor = &H8000000F
    #End If

    Me.caption = gProgressInfo.title
    offset = Me.height - (lbl6.top + lbl6.height)
    Set item = Me.Controls("lbl" & gProgressInfo.numStrings)
    Me.height = item.top + item.height + offset
    
    For i = 1 To 6
        Me.Controls("lbl" & i).caption = gProgressInfo.default(i)
    Next
End Sub

'Prevent click in closebox closing us
Private Sub UserForm_QueryClose(Cancel As Integer, CloseMode As Integer)
    If CloseMode = vbFormControlMenu Then
        Cancel = 1
    Else
        Cancel = 0
    End If
End Sub

