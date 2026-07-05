Attribute VB_Name = "MTSectionNum"
Attribute VB_Base = "0{0E7458BD-970C-4E3C-ACCD-DF89095BBBCC}{907CE7A0-FC3C-4B28-85FE-8DC39E9573A4}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False



'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTSectionNum.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
Option Explicit

'Enables/disables chapter fields
'When enabling, force section# to explicit, & default to 1
Private Sub cbChapter_Click()
    'turning on Chapter break inits section to an explicit "1"
    If cbChapter.value Then
        opt_ExplicitSecNum.value = True
        opt_IncrementSecNum.enabled = False
        edtSectionNumber.value = "1"

        opt_IncrementChapNum.enabled = True
        opt_ExplicitChapNum.enabled = True
        If opt_ExplicitChapNum.value Then
            EnableEditBox edtChapterNumber
        Else
            opt_IncrementChapNum.SetFocus
        End If
    Else
        opt_IncrementSecNum.enabled = True
        opt_IncrementChapNum.enabled = False
        opt_ExplicitChapNum.enabled = False
        DisableEditBox edtChapterNumber
    End If
End Sub

Private Sub opt_ExplicitSecNum_Click()
    EnableEditBox edtSectionNumber
End Sub

Private Sub opt_IncrementSecNum_Click()
    DisableEditBox edtSectionNumber
End Sub

Private Sub opt_ExplicitChapNum_Click()
    EnableEditBox edtChapterNumber
End Sub

Private Sub opt_IncrementChapNum_Click()
    DisableEditBox edtChapterNumber
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
    'Me.height = 2850
    'Me.width = 5355
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 168.5   '2856
    'Me.width = 277.25   '5352
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize

    'If Delete button is shown, must be modifying exisiting break
    If MTSecNum.gBreakDlg.showDelete Then
        Me.caption = "!0703Modify Chapter/Section Break"
    Else
        btn_Delete.visible = False
    End If

    'assign captions
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName
        'myControl.font.size = fontSize
        If myControl.name <> "edtChapterNumber" And myControl.name <> "edtSectionNumber" Then
            myControl.caption = MTLib.GetUserString(myControl.caption)
        End If
                #If Mac Then
                If TypeOf myControl Is MSForms.Frame Then
                        myControl.BackColor = &H8000000F
                End If
                #End If
    Next myControl

    'initialize the controls
    With MTSecNum.gBreakDlg
        If .isExplicitChapterNumber Then
            edtChapterNumber.value = .chapterNumber
            opt_ExplicitChapNum = True
        Else
            opt_IncrementChapNum = True
        End If
        cbChapter.value = .hasChapter
        cbChapter_Click     'have to call this explicitly
    
        opt_IncrementSecNum.value = Not .isExplicitSectionNumber
        If .isExplicitSectionNumber Then
            edtSectionNumber.value = .sectionNumber
            opt_ExplicitSecNum = True
        Else
            DisableEditBox edtSectionNumber
        End If
    
        .dlgCanceled = True
    End With
End Sub

'If data is OK, returns data in global structure
Private Sub OK_Click()
    Dim explicit As String

    'validate the explicit number(s)...
    If cbChapter.value And opt_ExplicitChapNum.value Then
        If MTLib.IsValidExplicitNumber(edtChapterNumber, explicit, Me.caption) Then
            MTSecNum.gBreakDlg.chapterNumber = explicit
        Else
            Exit Sub
        End If
    End If

    If opt_ExplicitSecNum Then
        If MTLib.IsValidExplicitNumber(edtSectionNumber, explicit, Me.caption) Then
            MTSecNum.gBreakDlg.sectionNumber = explicit
        Else
            Exit Sub
        End If
    End If

    MTSecNum.gBreakDlg.hasChapter = cbChapter.value
    MTSecNum.gBreakDlg.isExplicitChapterNumber = opt_ExplicitChapNum.value
    MTSecNum.gBreakDlg.isExplicitSectionNumber = opt_ExplicitSecNum.value

    MTSecNum.gBreakDlg.dlgCanceled = False
    unload Me
End Sub

Private Sub Cancel_Click()
    MTSecNum.gBreakDlg.dlgCanceled = True
    MTSecNum.gBreakDlg.delete = False
    unload Me
End Sub

Private Sub btn_Help_Click()
    If MTSecNum.gBreakDlg.showDelete Then
        OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#modify_chaptersection_break_dialog"
'        MTLib.MTHelpTopic hlpMSWDFormat_Equation_Section_Dialog
    Else
        OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#insert_chaptersection_break_dialog"
'        MTLib.MTHelpTopic hlpMSWDInsert_Equation_Section_Dialog
    End If
End Sub

Private Sub btn_Delete_Click()
    MTSecNum.gBreakDlg.dlgCanceled = False
    MTSecNum.gBreakDlg.delete = True
    unload Me
End Sub

