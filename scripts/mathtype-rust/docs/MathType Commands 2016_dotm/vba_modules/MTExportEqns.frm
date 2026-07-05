Attribute VB_Name = "MTExportEqns"
Attribute VB_Base = "0{ADAC66E4-3F9D-49FE-A27C-B57860251C17}{18C9CF3B-EFFF-4D52-AEE0-AB172E5EF2B2}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False


'MTExportEquationsDlg 5.0
'====================================================================
' (c) Copyright 2001-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTExportEqns.frm 2     7/02/14 3:10p Johns $
'====================================================================

'Also defined in MTExportEqns.bas
#If Win32 Then
Private Const kMaxFileTypeID As Long = 4
#Else
Private Const kMaxFileTypeID As Long = 5
#End If

Option Explicit

Private Sub btnBrowse_Click()
    Dim title As String
    Dim fileName As String
    Dim ext As String

    title = MTLib.GetUserString("!0644Export Equations - Choose Folder (As If Saving First File)")
    fileName = Strings.Space(256)
    ext = MTExportEquations.GetFileTypeExtension(cmbType.ListIndex)

    If MTSaveFileDialog(title, edtFolder.Text, _
        MTExportEquations.GetFileNameFromPattern(edtPattern.Text, Val(edtStart.Text)) & ext, _
        GetFileTypeDescription(cmbType.ListIndex), "*" & ext, fileName, 256) = 0 Then
        edtFolder.Text = MTLib.GetParentDirFromPath(Strings.Trim(fileName))
    End If
End Sub

Private Sub btnCancel_Click()
    MTExportEquations.gExportEquationDlgInfo.dlgCanceled = True
    unload Me
End Sub

Private Sub btnHelp_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#export_equations_dialog"
'    MTLib.MTHelpTopic hlpMSWDExport_Equations_Dialog
End Sub

Private Sub btnOK_Click()
    Dim stat As Long

    If VerifySettings() Then
        With MTExportEquations.gExportEquationDlgInfo
            .replace = cbReplace.value
            .deleteAll = cbDeleteAll.value
            .pattern = edtPattern.Text
            .fileType = GetFileTypeIndexFromControl
            .start = Val(edtStart.Text)
            .dlgCanceled = False
            .path = edtFolder.Text
            If optSelection.value Then
                .rangeType = mt_RANGE_SELECTION
            Else
                .rangeType = mt_RANGE_DOCUMENT
            End If
        End With
        unload Me
    End If
End Sub

'Return True if OK
Private Function VerifySettings() As Boolean
    Dim msg As String
    Dim stat As Long

    VerifySettings = False

    'check if export folder supplied
    If Len(edtFolder.Text) = 0 Then
        msg = MTLib.GetUserString("!0640Please enter a folder for exporting equations")
        MsgBox msg, vbCritical, Me.caption
        edtFolder.SetFocus
        Exit Function
    End If

    ' check for absolute path or simple filename - relative paths not allowed
    If InStr(edtFolder.Text, Application.PathSeparator) Or _
       InStr(edtFolder.Text, ":") Then
        If Not MTLib.IsVolumeAndAbsolutePath(edtFolder.Text) Then
                msg = MTLib.GetUserString("!0647Invalid path. You must enter a full, absolute path or a simple folder name. Relative paths are not permitted.")
                MsgBox edtFolder.Text & vbCrLf & msg, vbCritical, Me.caption
                edtFolder.SetFocus
            Exit Function
        End If
    Else
        ' ... build an absolute path using the document's directory
         edtFolder.Text = MTLib.GetParentDirFromPath(ActiveDocument.fullName) & Application.PathSeparator & edtFolder.Text
    End If

    'verify filename pattern
    If Len(edtPattern.Text) = 0 Then
        msg = MTLib.GetUserString("!0642Please enter a pattern for exporting equations")
        MsgBox msg, vbCritical, Me.caption
        edtPattern.SetFocus
        Exit Function
    ElseIf InStr(1, edtPattern.Text, "#", vbBinaryCompare) = 0 Then
        msg = MTLib.GetUserString("!0643The pattern must contain at least one '#' character.")
        MsgBox msg, vbCritical, Me.caption
        edtPattern.SetFocus
        Exit Function
    End If

    'verify First Number field
    If Not IsNumeric(edtStart.Text) Then
        msg = MTLib.GetUserString("!0646The First Number field must be numeric.")
        MsgBox msg, vbCritical, Me.caption
        edtStart.SetFocus
        Exit Function
    End If

    ' create directory if it doesn't exist
    If Not MTLib.CreateFolder(edtFolder.Text, Me.caption) Then
        Exit Function
    End If

    VerifySettings = True
End Function

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim fontName As String
    Dim fontSize As Long
    Dim prefix As String

    'assign the dialog's font & size according to the current language
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.font.name = fontName
    'Me.font.size = fontSize
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'Me.height = 5820
    'Me.width = 9345
    '#Else ' if Win
    'Me.height = 229.25  '4065
    'Me.width = 374.75   '7305
    '#End If

    'assign the captions according to the current language
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName
        'myControl.font.size = fontSize
        prefix = Strings.left$(myControl.name, 3)
        If prefix <> "edt" And prefix <> "cmb" Then
            myControl.caption = MTLib.GetUserString(myControl.caption)
        End If
        #If Mac Then
        If TypeOf myControl Is MSForms.Frame Then
            myControl.BackColor = &H8000000F
        End If
        #End If
    Next myControl

    With MTExportEquations.gExportEquationDlgInfo
        cbDeleteAll.value = .deleteAll
        cbReplace.value = .replace
        edtPattern.Text = .pattern
        LoadFileTypeList GetFileTypeIndexFromStruct
        edtStart.Text = Val(.start)
        edtFolder.Text = .path

        If .selType = wdSelectionIP Then
            optSelection.enabled = False
            optWhole.value = True
        Else
            optSelection.value = True
        End If
        .dlgCanceled = True
    End With
End Sub
Private Sub LoadFileTypeList(sel As Long)
    Dim i As Long
    Dim addItem As String
    For i = 0 To kMaxFileTypeID
        addItem = GetFileTypeDescription(i)
        If Len(addItem) > 0 Then
            cmbType.addItem addItem
        End If
    Next i

    cmbType.BoundColumn = 0
    If sel < cmbType.ListCount Then
        cmbType.ListIndex = sel
    Else
        cmbType.ListIndex = 0
    End If
End Sub

Private Function GetFileTypeIndexFromStruct() As Long
    #If Win32 Then
    GetFileTypeIndexFromStruct = MTExportEquations.gExportEquationDlgInfo.fileType
    #Else
    If MTExportEquations.gExportEquationDlgInfo.fileType <= 1 Then
        GetFileTypeIndexFromStruct = MTExportEquations.gExportEquationDlgInfo.fileType
    Else
        GetFileTypeIndexFromStruct = MTExportEquations.gExportEquationDlgInfo.fileType - 1
    End If
    #End If
End Function

Private Function GetFileTypeIndexFromControl() As Long
    #If Win32 Then
    GetFileTypeIndexFromControl = cmbType.ListIndex
    #Else
    If cmbType.ListIndex <= 1 Then
        GetFileTypeIndexFromControl = cmbType.ListIndex
    Else
        GetFileTypeIndexFromControl = cmbType.ListIndex + 1
    End If
    #End If
End Function

Private Function GetFileTypeDescription(id As Long) As String

    GetFileTypeDescription = ""
    Dim desc As String

    Select Case id
    Case kFTEPS_OSPICT
        #If Win32 Then
        desc = "!0630Encapsulated PostScript/WMF (*.eps)"
        #Else
        desc = "!0625Encapsulated PostScript/PICT"
        #End If
    Case kFTEPS_NONE
        #If Win32 Then
        desc = "!0632Encapsulated PostScript/none (*.eps)"
        #Else
        desc = "!0627Encapsulated PostScript/none"
        #End If
    Case kFTEPS_TIFF
        #If Win32 Then
        desc = "!0631Encapsulated PostScript/TIFF (*.eps)"
        #Else
        'desc = "!0626Encapsulated PostScript/TIFF"
        #End If
    Case kFTGIF
        #If Win32 Then
        desc = "!0633Graphics Interchange Format (*.gif)"
        #Else
        desc = "!0628Graphics Interchange Format"
        #End If
    Case kFTOSPICT
        #If Win32 Then
        desc = "!0634Window Metafile (*.wmf)"
        #Else
        desc = "!0629Macintosh PICT"
        #End If
    Case kFTPDF
        #If Win32 Then
        desc = "!0636Adobe PDF (*.pdf)"
        #Else
        desc = "!0635Adobe PDF"
        #End If
    End Select

    If Len(desc) > 0 Then
        GetFileTypeDescription = MTLib.GetUserString(desc)
    End If

End Function

