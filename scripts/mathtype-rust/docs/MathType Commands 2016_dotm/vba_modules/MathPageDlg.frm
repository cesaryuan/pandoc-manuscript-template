Attribute VB_Name = "MathPageDlg"
Attribute VB_Base = "0{7C39C4DE-E0B4-4003-ABA2-20F8F966459C}{ECE1A2C1-9E3F-4B3A-9F2C-ECECF3135E99}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False



'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MPOptions.frm 1     5/06/14 9:55a Jimm $
'=====================================================================

Option Explicit

Private gOKClicked As Boolean       'True if OK clicked
Private gInited As Boolean          'True once form has been inited
Private gValidExtensions As String
Private gGIFExtensions As String
Private gNewDoc As Boolean

Private Const kDefaultValidExtensions = ".htm;.html;.shtm;.shtml;.stm"

'Shows MP dialog, displaying options passed in.
'Saves options (if doc unlocked) if OK clicked.
'Returns True if user clicked OK.
Public Function ShowDialog() As Boolean
    InitDialog gOptions
    Me.Show
    ShowDialog = gOKClicked
    unload Me
End Function

'Init the dialog controls
Private Sub InitDialog(ByRef options As DialogOptions)
    Dim myControl As MSForms.control
    Dim settings
    Dim misc As String
    Dim fontName As String, fontSize As Long
    
    gInited = False
    
    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!001111"))
    'Me.height = 7785
    'Me.width = 10215
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 293.25  '5145
    'Me.width = 403.5    '7680
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize
    
    'assign the captions according to the current language
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        With myControl
            '.font.name = fontName
            '.font.size = fontSize
            If .name <> "edtDest" And .name <> "edtTitle" And _
                .name <> "comboTarget" And .name <> "lblDesc" Then
                .caption = MTLib.GetUserString(.caption)
            End If
        End With
        #If Mac Then
        If TypeOf myControl Is MSForms.Frame Then
            myControl.BackColor = &H8000000F
        End If
        #End If
    Next myControl

    edtTitle.Text = options.title
    edtDest.Text = options.fileName        'also sets OK state

    cbDisplay = options.openInBrowser
    cbMathZoom.value = options.mathZoom
    cbDefaults.value = False

    'get valid file extensions from registry else use defaults
    gGIFExtensions = GetPreference(HKEY_CURRENT_USER, mtreg_MT_MATHPAGE_KEY, "ValidExtensions")
    If gGIFExtensions = "" Then
        gGIFExtensions = kDefaultValidExtensions
    End If
    
    gNewDoc = Not MTLib.DocPropertyExists(ActiveDocument, kMP_HTMLDest)
    
    LoadMathMLTargets options
    
    'set IE compatibility option first, as MathML target may change it
    If options.ieOnly Then
        rbIE5Win.value = True
    Else
        rbAllBrowsers.value = True
    End If

    If options.useMathML Then
        rbMathML.value = True
        rbMathML_Click
    Else
        rbGIF.value = True
        rbGIF_Click
    End If

    gInited = True
End Sub

'Gets names of available MathML targets
Private Sub LoadMathMLTargets(ByRef options As DialogOptions)
    Dim i As Integer
    Dim target As MPMathMLTarget2
    Dim curTargetIndex As Long
    
    comboTarget.visible = False             'cause Change event to do nothing
    curTargetIndex = 0
    i = 0
    While GetCurrentMathMLTarget(i, target)
        comboTarget.addItem target.targetName 'ListIndex = i
        If target.targetName = options.mathMLtarget Then
            curTargetIndex = i
        End If
        i = i + 1
    Wend
    
    If comboTarget.ListCount = 0 Then
        comboTarget.addItem " "
    End If

    comboTarget.BoundColumn = 0             'Combo box values are ListIndex values
    comboTarget.ListIndex = curTargetIndex  'Set combo box's current value
    comboTarget.visible = True
End Sub

'Handles Browse button. Checks for Web Folder/FTP location, not allowed for now
Private Sub btnBrowse_Click()
    Dim dlg As Dialog
    Dim fullPath As String
    Dim curTitle As String  'doc's current title
    Dim stat As Long
    
    Set dlg = Dialogs(wdDialogFileSaveAs)
    With dlg
        .name = edtDest.Text
        .format = gHTMLFormatID
        
        'set title for 2000 which allows user to modify it
        curTitle = MTLib.GetDocTitle(ActiveDocument)
        If curTitle <> edtTitle.Text Then
            ActiveDocument.BuiltInDocumentProperties(wdPropertyTitle) = edtTitle.Text
        End If
        stat = .display()
        edtTitle.Text = MTLib.GetDocTitle(ActiveDocument)

        'check if user (accidentally?) selected a file format other than HTML output
        If .format <> gHTMLFormatID Then
            .name = MTLib.ReplaceExtension(.name, "htm")
        End If
        
        If stat = -1 Then
            'if ftp/http .Name will be correct...
            If Strings.left$(.name, 5) = "http:" Or Strings.left$(.name, 4) = "ftp:" Then
                fullPath = .name
            'if user typed a full path into the name field...
            ElseIf InStr(1, .name, Application.PathSeparator, vbTextCompare) > 0 Then
                fullPath = .name
            Else
                'if the name contains a space, Word will quote it - we don't want this
                If Strings.left$(.name, 1) = Strings.Chr(34) And Strings.right$(.name, 1) = Strings.Chr(34) Then
                    .name = Strings.Mid$(.name, 2, Len(.name) - 2)
                End If
                'fix up path if missing trailing path separator (e.g. "\" on Windows)
                If Strings.right$(FileSystem.CurDir, 1) = Application.PathSeparator Then
                    fullPath = FileSystem.CurDir & .name
                Else
                    fullPath = FileSystem.CurDir & Application.PathSeparator & .name
                End If
            End If
            
            If IsFileNameOK(fullPath) Then
                edtDest.Text = fullPath
            End If
        End If

        'Forces our dialog to be active
        If Len(edtDest.Text) > 0 Then
            btnOK.enabled = True
            btnOK.SetFocus
        Else
            btnCancel.SetFocus
        End If
    End With
End Sub

'Returns True if filename passes validation
Private Function IsFileNameOK(name As String) As Boolean
    IsFileNameOK = False
    
    'check for bad characters
    Dim i As Long
    Dim badchars As String
    badchars = ""
    
    #If Win32 Then
    Const kBadChars = ":/*?<>|"""
    #Else
    Const kBadChars = "*?<>|"""
    #End If
    
    For i = 1 To Len(name)
        Dim c As String
        Dim ch As Long
        c = Strings.Mid$(name, i, 1)
        ch = Asc(c)
        If ch < 32 Or ch > 126 Or InStr(kBadChars, c) > 0 Then
            #If Win32 Then
            If i <> 2 Or c <> ":" Then
                badchars = badchars & c
            End If
            #Else
            badchars = badchars & c
            #End If
        End If
    Next
    If Len(badchars) Then
        MsgBox MTLib.GetUserString2("0924", "0925", "MathPage does not support the following characters in the filename: " & badchars), vbCritical, kMPName
        Exit Function
    End If
    
    ' check for ftp or http (not supported yet)
    If Strings.left$(name, 3) = "ftp" Then
        MsgBox MTLib.GetUserString("!0914FTP locations not currently supported. Please choose another location."), vbCritical, kMPName
        Exit Function
    ElseIf Strings.left$(name, 4) = "http" Then
        MsgBox MTLib.GetUserString("!0915Web folders not curently supported. Please choose another location."), vbCritical, kMPName
        Exit Function
    End If
    
    ' handle absolute paths
    If InStr(name, Application.PathSeparator) Or _
       InStr(name, ":") Then
        
        ' check for valid absolute path - relative paths not allowed
        If Not MTLib.IsVolumeAndAbsolutePath(name) Then
            MsgBox MTLib.GetUserString("!0918Invalid path. You must enter a full, absolute path or a simple filename. Relative paths are not permitted."), _
                vbCritical, kMPName
            Exit Function
        End If
        
        ' create directory if it doesn't exist
        If Not MTLib.CreateFolder(MTLib.GetParentDirFromPath(name), Me.caption) Then
            Exit Function
        End If

    End If
    
    ' check for extension
    If InStrR(MTLib.GetFileNameFromPath(name), ".") <= 1 Then
        MsgBox MTLib.GetUserString("!0917The file name is missing a valid name or extension. Please choose another file name."), vbCritical, kMPName
        Exit Function
    End If
    
    'can't have files sharing same supporting files folder, so test for 2 files w/same name
    IsFileNameOK = IsFileNameUnique(name)

End Function

'Returns False if filename exists (or will exist) with multiple extensions
'Due to supporting files folder, Word doesn't allow this.
Private Function IsFileNameUnique(name As String) As Boolean
    Dim tempName As String
    Dim extSep As Integer
    Dim fileName As String
    Dim fileList As New Collection
    Dim baseName As String
    Dim result As Long
    Dim parentDir As String
    Dim ext As String
    Dim baseName1 As String
    Dim ext1 As String
    Dim msg As String
    Dim i As Long

    IsFileNameUnique = True
    tempName = name
    extSep = InStrR(tempName, ".")
    
    'If file has .htm extension, look for .html, and vice-versa
    If extSep = 0 Then
        'bail w/success if file doesn't have an extension
        Exit Function
    End If

    'search for the clashing files...
    tempName = Strings.left$(tempName, extSep - 1) & ".*"
    On Error GoTo badfilename
    baseName = MTLib.GetFileNameFromPath(name)
    extSep = InStrR(baseName, ".")
    ext = Strings.right$(baseName, Len(baseName) - extSep)
    baseName = Strings.left$(baseName, extSep - 1)
    #If Win32 Then
    fileName = dir(tempName)
    #Else
    parentDir = MTLib.GetParentDirFromPath(name)
    If Strings.right$(parentDir, 1) <> Application.PathSeparator Then
        parentDir = parentDir & Application.PathSeparator
    End If
    fileName = dir(parentDir)
    #End If
    While fileName <> ""
        extSep = InStrR(fileName, ".")
        If extSep > 1 Then
            baseName1 = Strings.left$(fileName, extSep - 1)
        Else
            baseName1 = fileName
        End If
        If extSep > 0 And extSep < Len(fileName) Then
            ext1 = Strings.right$(fileName, Len(fileName) - extSep)
        Else
            ext1 = ""
        End If
        If (Strings.UCase$(fileName) <> Strings.UCase$(ActiveDocument.name)) And _
            (Strings.UCase$(baseName) = Strings.UCase$(baseName1)) And (Strings.UCase$(ext) <> Strings.UCase$(ext1)) Then
            fileList.Add (fileName)
        End If
        fileName = dir
    Wend
    
    'for a non-null list allow the user to delete them and proceed or cancel
    If fileList.count > 0 Then
        msg = MTLib.GetUserString("!0920Word does not allow you to save a file as a web page when there are duplicate files in the directory with the same name (but different extensions). The following file(s) exist in this directory:") & vbNewLine & vbNewLine
        For i = 1 To fileList.count
            msg = msg & "       " & fileList(i) & vbNewLine
        Next
        msg = msg & vbNewLine & _
               MTLib.GetUserString("!0921The above file(s) must be deleted in order to save this document as a web page. Press OK to delete the above file(s) and continue, or press Cancel to choose another filename or directory.")
        result = MsgBox(msg, vbOKCancel + vbExclamation, kMPName)
        IsFileNameUnique = (result = vbOK)
        If result = vbOK Then
            parentDir = MTLib.GetParentDirFromPath(name)
            For i = 1 To fileList.count
                Kill parentDir & Application.PathSeparator & fileList(i)
            Next
        End If
    End If
    Exit Function
badfilename:
    IsFileNameUnique = False
    MsgBox MTLib.GetUserString("!1008The filename is invalid, please check the whole filename and try again."), _
        vbCritical, kMPName
End Function

Private Sub btnCancel_Click()
    gOKClicked = False
    unload Me
End Sub

'Returns name of specified MathML target
Private Function GetCurrentMathMLTarget( _
    ByVal index As Integer, _
    ByRef target As MPMathMLTarget2) As Boolean

    On Error Resume Next
    
    GetCurrentMathMLTarget = False
    target.version = 1
    target.targetName = Strings.Space(64)
    target.nameMax = 64
    target.targetDesc = Strings.Space(256)
    target.descMax = 256
    target.extensions = Strings.Space(64)
    target.extMax = 64
    Dim stat As Long
    stat = MPEnumMathMLTarget2(index, target)
    If stat = 1 Then
        target.targetName = CTrim(target.targetName)
        target.targetDesc = CTrim(target.targetDesc)
        target.extensions = CTrim(target.extensions)
        GetCurrentMathMLTarget = True
    End If
End Function

'Saves Boolean value to defaults location in registry
Private Sub SaveBoolDefault(ByRef name As String, value As Boolean)
    Dim textValue As String
    If value Then
        textValue = "1"
    Else
        textValue = "0"
    End If
    SetPreference HKEY_CURRENT_USER, mtreg_MT_MATHPAGE_KEY, name, textValue
End Sub

Private Sub btnHelp_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#publish_to_mathpage_dialog"
'    MTLib.MTHelpTopic hlpMSWDExport_MathPage_Dialog
End Sub

Private Sub btnOK_Click()
    If IsTargetOK Then
        If IsFileExtensionOK Then
            If SaveProperties(gOptions) Then
                gOKClicked = True
                unload Me
            End If
        End If
    End If
End Sub

'Returns True if current target is compatible with current version of Word
Private Function IsTargetOK() As Boolean
    Dim target As MPMathMLTarget2

    IsTargetOK = True
    If rbGIF Then Exit Function
    GetCurrentMathMLTarget comboTarget.value, target
    If target.minWordVer > gAppVersion Then
        MsgBox MTLib.GetUserString("!1024This target is not supported in this version of Word. Please choose another target."), _
            vbOKOnly + vbExclamation, kMPName
        IsTargetOK = False
    End If
End Function

'Returns True if file extension is in the list of valid extensions
Private Function IsFileExtensionOK() As Boolean
    Dim dotExtensionSem As String
    Dim dotPos As Long
    Dim result As Long
    Dim extensions As String
    Dim targetChanged As Boolean
    Dim target As MPMathMLTarget2
    
    IsFileExtensionOK = True
    
    'see if user selected a new target
    targetChanged = False
    If (gOptions.useMathML <> rbMathML.value) Then
        targetChanged = True
    ElseIf rbMathML.value Then
        GetCurrentMathMLTarget comboTarget.value, target
        If (gOptions.mathMLtarget <> target.targetName) Then
            targetChanged = True
        End If
    End If
    
    'check extension on new docs or when target changed only
    If targetChanged Or edtDest.Text <> gOptions.fileName Then
        dotPos = InStrR(edtDest.Text, ".")
        If dotPos > 0 Then
            dotExtensionSem = Strings.right$(edtDest.Text, Len(edtDest.Text) - dotPos + 1) & ";"
        End If
        extensions = gValidExtensions & ";"
        If dotExtensionSem = "" Or (InStr(1, Strings.UCase$(extensions), Strings.UCase$(dotExtensionSem)) = 0) Then
            result = MsgBox(MTLib.GetUserString("!0922Valid extensions for this target are: ") & gValidExtensions & vbNewLine & _
                            MTLib.GetUserString("!0923Do you wish to continue?"), _
                vbYesNo + vbExclamation, kMPName)
            IsFileExtensionOK = (result = vbYes)
        End If
    End If
End Function

'only save properties if necessary
'returns True if OK to exit & process the document
'otherwise displays error and returns False
Private Function SaveProperties(ByRef options As DialogOptions) As Boolean
    Dim dirty As Boolean
    Dim Doc As Document
    Dim target As MPMathMLTarget2
    Dim attribs As Integer

    SaveProperties = True
    dirty = False
    
    Set Doc = ActiveDocument
    'make sure output name is different from source filename
    If Doc.fullName = edtDest.Text Then
        MsgBox "You have chosen the same filename as the current document. Please select a different name and try again.", _
            vbCritical + vbOKOnly, kMPName
        SaveProperties = False
        Exit Function
    End If

    If Not IsFileNameOK(edtDest.Text) Then
        SaveProperties = False
        Exit Function
    End If
    
    If InStr(edtDest.Text, Application.PathSeparator) = 0 Then
        ' ... build an absolute path using the document's directory
         edtDest.Text = MTLib.GetParentDirFromPath(ActiveDocument.fullName) & Application.PathSeparator & edtDest.Text
    End If
    
    With options
        If edtTitle.Text <> .title Then
            .title = edtTitle.Text
            dirty = True
        End If
        If (edtDest.Text <> .fileName) Or (Not DocPropertyExists(Doc, kMP_HTMLDest)) Then
            .fileName = edtDest.Text
            dirty = True
        End If
        If (.openInBrowser <> cbDisplay.value) Or (Not DocPropertyExists(Doc, kMP_OpenInBrowser)) Then
            .openInBrowser = cbDisplay.value
            dirty = True
        End If
        If (.useMathML <> rbMathML.value) Or (Not DocPropertyExists(Doc, kMP_UseMathML)) Then
            .useMathML = rbMathML.value
            dirty = True
        End If
        If (.mathZoom <> cbMathZoom.value) Or (Not DocPropertyExists(Doc, kMP_MathZoom)) Then
            .mathZoom = cbMathZoom.value
            dirty = True
        End If
        If (.ieOnly <> rbIE5Win.value) Or (Not DocPropertyExists(Doc, kMP_IEOnly)) Then
            .ieOnly = rbIE5Win.value
            dirty = True
        End If
        GetCurrentMathMLTarget comboTarget.value, target
        If (.mathMLtarget <> target.targetName) Or _
            (Not DocPropertyExists(Doc, kMP_MathMLTarget)) Then
            .mathMLtarget = target.targetName
            dirty = True
        End If
    End With

    'if one or more properties was changed, save all of them
    If dirty And Not Doc.ReadOnly Then
        Doc.BuiltInDocumentProperties("Title").value = options.title
        WriteStringDocProperty Doc, kMP_HTMLDest, options.fileName
        WriteStringDocProperty Doc, kMP_MathMLTarget, options.mathMLtarget
        WriteBoolDocProperty Doc, kMP_OpenInBrowser, options.openInBrowser
        WriteBoolDocProperty Doc, kMP_UseMathML, options.useMathML
        WriteBoolDocProperty Doc, kMP_MathZoom, options.mathZoom
        WriteBoolDocProperty Doc, kMP_IEOnly, options.ieOnly
        
        'wrap in error handler to silently catch locked media errors etc.
        On Error Resume Next
        Doc.saved = False
        Doc.Save
        Doc.saved = True
        On Error GoTo 0
    End If

    'save defaults...
    If cbDefaults.value Then
        SaveDefaults
    End If
End Function

'Saves settings to registry to use as defaults
Private Sub SaveDefaults()
    Dim target As MPMathMLTarget2

    SaveBoolDefault kMP_OpenInBrowser, cbDisplay.value
    SaveBoolDefault kMP_UseMathML, rbMathML.value
    SaveBoolDefault kMP_MathZoom, cbMathZoom.value
    SaveBoolDefault kMP_IEOnly, rbIE5Win.value
    If GetCurrentMathMLTarget(comboTarget.value, target) Then
        SetPreference HKEY_CURRENT_USER, mtreg_MT_MATHPAGE_KEY, kMP_MathMLTarget, target.targetName
    End If
End Sub

'Handles click in MathML target list
Private Sub comboTarget_Change()
    Dim target As MPMathMLTarget2

    If comboTarget.visible Then     'prevents running before fully loaded
        If GetCurrentMathMLTarget(comboTarget.value, target) Then
            
            If target.browser = 0 Then 'Win IE5+
                rbIE5Win.enabled = True
                rbIE5Win.value = True
                rbAllBrowsers.enabled = False
            ElseIf target.browser = 1 Then 'All Browsers
                rbAllBrowsers.enabled = True
                rbAllBrowsers.value = True
                rbIE5Win.enabled = False
            Else 'default or -1: let user select
                rbAllBrowsers.enabled = True
                rbIE5Win.enabled = True
            End If
            lblDesc.caption = target.targetDesc
        
            'save list of valid extensions for later validation
            CheckExtension target.extensions
            
        End If
        
    End If
End Sub

'Enable OK only if we have some data...
Private Sub edtDest_Change()
    btnOK.enabled = (Len(edtDest.Text) > 0)
End Sub

'Enable MathZoom checkbox & both browser options, & disable MathML targets list
Private Sub rbGIF_Click()
    cbMathZoom.enabled = True
    comboTarget.enabled = False
    rbIE5Win.enabled = True
    rbAllBrowsers.enabled = True
    lblDesc.caption = MTLib.GetUserString("!0916Uses GIF images for equations and some symbols. Supported on all platforms.")
    CheckExtension gGIFExtensions
End Sub

Private Sub CheckExtension(extensions As String)
    Dim defaultExtension As String
    Dim pos As Long
    Dim extension As String
    
    'save valid extensions for later validation
    gValidExtensions = extensions
    
    ' if document extension not in list...
    pos = InStrR(edtDest.Text, ".")
    If pos > 0 Then
        extension = Strings.right$(edtDest.Text, Len(edtDest.Text) - pos + 1) & ";"
    End If
    extensions = extensions & ";"
    If Len(extension) = 0 Or InStr(1, Strings.UCase$(extensions), Strings.UCase$(extension)) = 0 Then
        '...set the default extension
        pos = InStr(1, extensions, ";")
        If pos > 0 Then
            defaultExtension = Strings.Mid$(extensions, 2, pos - 2)
            edtDest.Text = MTLib.ReplaceExtension(edtDest.Text, defaultExtension)
        End If
    End If
End Sub

'Disable MathZoom checkbox & enable MathML targets list
Private Sub rbMathML_Click()
    cbMathZoom.enabled = False
    comboTarget.enabled = True
    comboTarget_Change
    If gInited Then
        ShowMathMLHelpInfo
    End If
End Sub

'Shows Help tip for MathML targets first time in
Private Sub ShowMathMLHelpInfo()
    If GetPreference(HKEY_CURRENT_USER, mtreg_MT_MATHPAGE_KEY, mtreg_MT_WORD_SEEN_MATHMLHELP) <> "1" Then
        MsgBox MTLib.GetUserString("!0919MathType Help contains detailed information about the various MathML targets. Close this dialog and then click the Help button."), _
            vbInformation + vbOKOnly, kMPName
        SetPreference HKEY_CURRENT_USER, mtreg_MT_MATHPAGE_KEY, mtreg_MT_WORD_SEEN_MATHMLHELP, "1"
    End If
End Sub


