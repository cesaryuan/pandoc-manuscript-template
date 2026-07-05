Attribute VB_Name = "MTEqnNumFormatDlg"
Attribute VB_Base = "0{831F26D9-8C71-4017-871B-777F4B199376}{37C7C7DF-4368-4743-8D94-3C8C37C48BDF}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False




'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTEqnNumFormatDlg.frm 1     5/06/14 9:55a Jimm $
'=====================================================================
Option Explicit

Private Const kBreakToken As String = "#"
Private Const kBreakCodes As String = "CSE"

'order matches order in lists
Private Const kEncStartChars As String = "([{<"
Private Const kEncEndChars As String = ")]}>"
Private Const kBreakFormats As String = "1IiAa"

Private gInInit As Boolean

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim fontName As String
    Dim fontSize As Long
    Dim namePrefix As String

    gInInit = True

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!00118"))
    'Me.height = 9810
    'Me.width = 7410
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 380     '7080
    'Me.width = 320.75   '6225
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize

    'assign the captions according to the current language
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName
        'myControl.font.size = fontSize
        namePrefix = Strings.left$(myControl.name, 3)
        If namePrefix <> "edt" And namePrefix <> "cmb" And namePrefix <> "txt" _
            And myControl.name <> "frmDivider" Then
            myControl.caption = MTLib.GetUserString(myControl.caption)
        End If
        #If Mac Then
        If TypeOf myControl Is MSForms.Frame Then
            myControl.BackColor = &H8000000F
        End If
        #End If
    Next myControl

    'init the format lists
    SetFormatList Me.cmbChapter
    SetFormatList Me.cmbSection
    SetFormatList Me.cmbEquation
    SetEnclosureList Me.cmbEnclosure

    With MTEqnNumFormat.gEqnNumFormatDlgInfo
        'set the format settings
        If InitFormatSettings(.format) Then
            optSimple.value = True
            .customFormat = False
        Else
            .customFormat = True
        End If
        If .customFormat Then
            SetDefaultFormatSettings
            edtFormat.Text = .format
            optAdvanced.value = True
        Else
        End If

        cbSetCurrent.value = True

        rbCurSelection.value = False
        If .selectionType = wdSelectionIP Then
            rbCurSelection.enabled = False
            rbWholeDoc.value = True
        Else
            rbCurSelection.value = True
        End If

        cbAutoFieldUpdate.value = Not .deferUpdate
        cbInsertNumWarning.value = Not .suppressEqnNumWarning
        cbInsertRefWarning.value = Not .suppressEqnRefWarning

        .dlgCanceled = True
    End With

    'Set the sample text
    gInInit = False
    UpdatePreview

    btnOK.SetFocus

End Sub

'"(#S1.#C1)", i.e. "(1.1)"
Private Sub SetDefaultFormatSettings()
    cbChapter.value = False
    cbSection.value = True
    cbEquation.value = True
    cbSeparator.value = True
    edtSeparator.Text = "."
    cbEnclosure.value = True
End Sub

'sets format settings as determined by the format string
'returns true if format is simple, false if advanced/custom
Private Function InitFormatSettings(format As String)
    Dim pos1 As Long, pos2 As Long
    Dim token As String
    Dim break As Long
    Dim breakStyle As Long

    InitFormatSettings = False 'assume custom until we get a break code

    'get enclosure
    pos1 = InStr(1, kEncStartChars, Strings.left$(format, 1), vbBinaryCompare)
    If pos1 = 0 Then
        cbEnclosure.value = False
    Else
        If Strings.right$(format, 1) = Strings.Mid$(kEncEndChars, pos1, 1) Then
            cbEnclosure.value = True
            cmbEnclosure.ListIndex = pos1 - 1
            pos1 = 1
        End If
    End If

    'loop through format string, parsing all break codes and formats
    pos1 = pos1 + 1
    While pos1 <= Len(format)

        'find next break token (#)
        pos2 = InStr(pos1, format, kBreakToken, vbBinaryCompare)
        If pos2 <= 0 Then
            If (cbEnclosure = False) Or (pos1 < Len(format)) Then
                'extra stuff at end; custom
                InitFormatSettings = False
            End If
            Exit Function
        End If

        'get separator char or string
        token = Strings.Mid$(format, pos1, pos2 - pos1)
        If (cbSeparator.value = False) And Len(token) > 0 Then
            cbSeparator.value = True
            edtSeparator.Text = token
        Else
            If edtSeparator.Text <> token Then
                'different separator; custom
                InitFormatSettings = False
                Exit Function
            End If
        End If

        'get break code
        pos2 = pos2 + 1
        token = Strings.Mid$(format, pos2, 1)
        break = InStr(1, kBreakCodes, token, vbBinaryCompare)
        If break <= 0 Then
            'unknown break code; custom
            InitFormatSettings = False
            Exit Function
        Else
            'get break style
            pos2 = pos2 + 1
            token = Strings.Mid$(format, pos2, 1)
            breakStyle = InStr(1, kBreakFormats, token, vbBinaryCompare)
            If breakStyle <= 0 Then
                'unknown break style; custom
                InitFormatSettings = False
                Exit Function
            Else
                Select Case break
                Case 1
                    'C=chapter break
                    cbChapter = True
                    cmbChapter.ListIndex = breakStyle - 1
                Case 2
                    'S=section break
                    cbSection = True
                    cmbSection.ListIndex = breakStyle - 1
                Case 3
                    'E=equation break
                    cbEquation = True
                    cmbEquation.ListIndex = breakStyle - 1
                End Select
                InitFormatSettings = True 'got a valid break code
                pos2 = pos2 + 1
            End If
        End If

        pos1 = pos2

    Wend
    'should never get here
End Function

'Do NOT change order, other code depends upon it
Private Sub SetFormatList(combo As MSForms.ComboBox)
    combo.addItem "1,2,3,..."
    combo.addItem "I,II,III,..."
    combo.addItem "i,ii,iii,..."
    combo.addItem "A,B,C,..."
    combo.addItem "a,b,c,..."
    combo.BoundColumn = 0
    combo.ListIndex = 0
End Sub

'Do NOT change order, other code depends upon it
Private Sub SetEnclosureList(combo As MSForms.ComboBox)
    combo.addItem "( )"
    combo.addItem "[ ]"
    combo.addItem "{ }"
    combo.addItem "< >"
    combo.BoundColumn = 0
    combo.ListIndex = 0
End Sub

Private Sub btnCancel_Click()
    MTEqnNumFormat.gEqnNumFormatDlgInfo.dlgCanceled = True
    unload Me
End Sub

Private Sub btnOK_Click()
    'get the return values
    With MTEqnNumFormat.gEqnNumFormatDlgInfo
        .changeFuture = cbSetCurrent.value
        .changeSelected = rbCurSelection.value
        .changeWholeDoc = rbWholeDoc.value

        .format = edtFormat.Text
        .customFormat = optAdvanced.value
        .deferUpdate = Not cbAutoFieldUpdate.value
        .suppressEqnNumWarning = Not cbInsertNumWarning.value
        .suppressEqnRefWarning = Not cbInsertRefWarning.value
        .useAsDefaults = cbDefaults.value
        .dlgCanceled = False
    End With

    unload Me
End Sub


Private Sub btnHelp_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#format_equation_numbers_dialog"
'    MTLib.MTHelpTopic hlpMSWDEquation_Number_Format_Dialog
End Sub

'Converts format string into preview (Chapter, Section & Equation# = 1)
Private Function GetPreview(format As String) As String
    Dim preview As String
    Dim start As Long, pos As Long

    start = 1
    pos = InStr(start, format, kBreakToken, vbBinaryCompare)
    While (pos > 0)
        preview = preview & Strings.Mid$(format, start, pos - start)
        preview = preview & GetFormattedNumber(format, pos)
        start = pos
        pos = InStr(start, format, kBreakToken, vbBinaryCompare)
    Wend
    GetPreview = preview & Strings.Mid$(format, start)
End Function

'Returns value formatted according to next char in format string (1,I,i,A,a).
'Returns # if none of these, also advances pos accordingly
Private Function GetFormattedNumber(format As String, ByRef pos As Long) As String
    Dim token As String

    pos = pos + 1
    token = Strings.Mid$(format, pos, 1)
    If InStr(1, "CSE", token, vbBinaryCompare) > 0 Then
        pos = pos + 1
        token = Strings.Mid$(format, pos, 1)
        Select Case token
        Case "1", "I", "i", "A", "a"
            GetFormattedNumber = token
            pos = pos + 1
        Case Else
            GetFormattedNumber = kBreakToken
            pos = pos - 1
        End Select
    Else
        GetFormattedNumber = kBreakToken
    End If
End Function

'Returns format string based on custom setting and chosen items
'If simple, updates value on format field too
Private Function GetFormat() As String
    Dim format As String

    If optAdvanced.value Then
        GetFormat = edtFormat.Text
    Else
        format = GetEnclosureSetting(True)
        If cbChapter.value Then
            format = format & "#C" & GetPreviewNumber(cmbChapter.value)
            If cbSection.value Or cbEquation.value Then
                format = format & GetSeparatorSetting()
            End If
        End If
        If cbSection.value Then
            format = format & "#S" & GetPreviewNumber(cmbSection.value)
            If cbEquation.value Then
                format = format & GetSeparatorSetting()
            End If
        End If
        If cbEquation.value Then
            format = format & "#E" & GetPreviewNumber(cmbEquation.value)
        End If
        format = format & GetEnclosureSetting(False)
        edtFormat.Text = format
        GetFormat = format
    End If
End Function

'Returns left or right enclosure
Private Function GetEnclosureSetting(start As Boolean) As String
    Dim encArray As String
    GetEnclosureSetting = ""
    If cbEnclosure.value Then
        If start Then
            encArray = kEncStartChars
        Else
            encArray = kEncEndChars
        End If
        GetEnclosureSetting = Strings.Mid$(encArray, cmbEnclosure.ListIndex + 1, 1)
    End If
End Function

'Returns number for preview based on index of pull-down list
Private Function GetPreviewNumber(index As Long) As String
    GetPreviewNumber = Strings.Mid$(kBreakFormats, index + 1, 1)
End Function

Private Function GetSeparatorSetting() As String
    If cbSeparator.value Then
        GetSeparatorSetting = edtSeparator.Text
    End If
End Function

Private Sub cbChapter_Click()
    cmbChapter.enabled = cbChapter.value
    UpdatePreview
End Sub

Private Sub cbSection_Click()
    cmbSection.enabled = cbSection.value
    UpdatePreview
End Sub

Private Sub cbEquation_Click()
    cmbEquation.enabled = cbEquation.value
    UpdatePreview
End Sub

Private Sub cbEnclosure_Click()
    cmbEnclosure.enabled = cbEnclosure.value
    UpdatePreview
End Sub

Private Sub cbSeparator_Click()
    If cbSeparator.value Then
        EnableEditBox edtSeparator
    Else
        DisableEditBox edtSeparator
    End If
    UpdatePreview
End Sub

'Enable/disable simple items & their lists
Private Sub EnableFormatItems(state As Boolean)
    cbChapter.enabled = state
    cbSection.enabled = state
    cbEquation.enabled = state
    cbEnclosure.enabled = state
    cbSeparator.enabled = state
    If state Then
        cmbChapter.enabled = cbChapter.value
        cmbSection.enabled = cbSection.value
        cmbEquation.enabled = cbEquation.value
        cmbEnclosure.enabled = cbEnclosure.value
        If cbSeparator.value Then
            EnableEditBox edtSeparator
        End If
    Else
        cmbChapter.enabled = False
        cmbSection.enabled = False
        cmbEquation.enabled = False
        cmbEnclosure.enabled = False
        DisableEditBox edtSeparator
    End If
End Sub

'Following are called when user makes a new format choice (typing or choosing)
Private Sub edtFormat_Change()
    'only update if in simple mode - otherwise get recursive calling on Macs!
    If optAdvanced.value Then
        UpdatePreview
    End If
End Sub

Private Sub cmbChapter_Change()
    UpdatePreview
End Sub

Private Sub cmbSection_Change()
    UpdatePreview
End Sub

Private Sub cmbEquation_Change()
    UpdatePreview
End Sub

Private Sub cmbEnclosure_Change()
    UpdatePreview
End Sub

Private Sub edtSeparator_Change()
    UpdatePreview
End Sub

Private Sub optAdvanced_Click()
    edtFormat.enabled = True
    edtFormat.selStart = 0
    edtFormat.SelLength = Len(edtFormat.Text)
    edtFormat.SetFocus
    EnableFormatItems False
End Sub

Private Sub optSimple_Click()
    edtFormat.enabled = False
    EnableFormatItems True
    optSimple.SetFocus
    UpdatePreview
End Sub

Private Sub UpdatePreview()
    Dim format As String

    If Not gInInit Then
        format = GetFormat()
        txtPreview.caption = GetPreview(format)
    End If
End Sub







