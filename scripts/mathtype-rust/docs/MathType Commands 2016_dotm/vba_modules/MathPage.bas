Attribute VB_Name = "MathPage"
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MathPage.bas 106   5/06/14 9:54a Jimm $
'=====================================================================
Option Explicit

Type DialogOptions
    title As String             'document's title
    fileName As String          'full path of filename to create
    ieOnly As Boolean           'True to generate IE5+ Windows only-compatible output
    openInBrowser As Boolean    '1 to open doc. in browser
    antiAlias As Boolean        'True to anti-alias GIFs
    useMathML As Boolean        'True to use MathML, False to use GIFs
    mathMLtarget As String      'MathML target, ignored if useMathML is false
    mathZoom As Boolean
    insertTranslationError As String 'True to insert error message after equation
End Type

'Global variables
Public gOptions As DialogOptions
Public gHTMLFormatID As Long        'HTML converterID (diff. for Word97/2000)
Public gAppVersion As Long          'major app version

Public Const kMPName As String = "MathPage"

'Error codes used in thrown errors (must be in range 513 - 65535)
Private Const kInitWLLError As Long = 600
Private Const kPostProcessDocError As Long = 601

'Document Props/Reg. Values
'paths aren't x-platform...
#If Win32 Then
Public Const kMP_HTMLDest           As String = "MP_HTMLDest"
#End If
#If Mac Then
Public Const kMP_HTMLDest           As String = "MP_HTMLDestMac"
#End If
Public Const kMP_IEOnly             As String = "MP_IEOnly"
Public Const kMP_OpenInBrowser      As String = "MP_OpenInBrowser"
Public Const kMP_UseMathML          As String = "MP_UseMathML"
Public Const kMP_MathMLTarget       As String = "MP_MathMLTarget"
Public Const kMP_MathZoom           As String = "MP_MathZoom"
Public Const kMP_KeepWordHTMLFile   As String = "MP_KeepWordHTMLFile"
Public Const kMP_TranslationError   As String = "MP_TranslationError"

'default ext. used by Word for subdocs
Private Const kDefaultHTMLExtension As String = "htm"

'Private 'member' variables
Private mDocument As Document       'the doc
Private mSymbolCount As Long        'count of symbols processed
Private mNormalText As String       'InsertSymbol's localized string for 'normal text'
Private mFolderSuffix As String     'Suffix used for support folder, eg "_Files"
Private mHasFloatingEquations As Boolean   'True if floating objects in doc
Private mHasMissingFonts As Boolean   'True if fonts missing in doc
Private mHasTranslatorErrors As Boolean   'True if translator errors occurred
Private mKeepWordHTMLFile As Boolean 'True to keep the Word's .htm file
Private mOldBookmark() As String    'bookmark arrays to resolve dups. for same eqn num
Private mNewBookmark() As String
Private mNumBookmarks As Long
Private mOldShowFieldCodes As Boolean
Private mOldSmartCutPaste As Boolean
Private mOldTrackRevisions As Boolean
Private mOldPagination As Boolean
Private mOldSelection As SelInfo
Private mResult As Long
Private mExtension As String
Private mSubDocNames() As String
Private mTempGIFPath As String
Private mShowWarnings As Boolean    'true to display warning dialogs

'values for subscript/superscript
Private Const kNoSubSuper As Long = 0
Private Const kSubScript As Long = 1
Private Const kSuperScript As Long = 2

'ratio by which to shrink size of subscript/superscript symbols
Private Const kSuperSubscriptRatio As Single = 0.66

'styles that provide manual overrides for equation processing
Private Const kInlineEquationStyle  As String = "MP_InlineEquation"
Private Const kDisplayEquationStyle As String = "MP_DisplayEquation"

'struct for saving & restoring selection
Type SelInfo
    storyType As Long
    storyPos As Long
    start As Long
    end As Long
    viewType As Long
End Type

'tag strings
Private Const ktagEntityStart   As String = "[!DSTAG ENTITY CHAR="
Private Const ktagDisplayTable  As String = "[!DSTAG DISPLAYTABLE /]"
Private Const ktagEnd           As String = "/]"

'name of empty GIF file
Private Const kEmptyGIF As String = "empty.gif"
Private Const kTempGIF As String = "temp.gif"

'amount of padding around equations that gets trimmed when GIFs are created
Private Const kEqnPadding As Single = 4

'progress ids
Private Const kPROGRESS_SYMBOL As Long = 1
Private Const kPROGRESS_EQUATION As Long = 2
Private Const kPROGRESS_EQNNUM As Long = 3
Private Const kPROGRESS_HTML As Long = 4

'Entry point
Sub MP_ExportTo()
    ExportCurDocToHTML
End Sub

'Entry point: Exports current document to HTML (i.e. with GIFs)
Sub ExportCurDocToHTML()
    Dim Doc As Document
    Dim oldBS As Boolean

    If IsWLLVersionOK() Then
        'save/set this setting here as we may save cur doc before processing
        oldBS = Application.options.BackgroundSave
        Application.options.BackgroundSave = False

        Init
        If MTLib.IsDocumentOpen(kMPName) = False Then
            Application.options.BackgroundSave = oldBS
            Exit Sub
        End If

        'check if this command is allowed in the current view
        If MTLib.IsCurrentViewOK(kMPName) = False Then
            Exit Sub
        End If

        Set Doc = ActiveDocument.ActiveWindow.Document
        If DocumentOK(Doc) Then
            'get options from custom doc. properties
            GetOptions gOptions

            If MathPageDlg.ShowDialog() Then
                DoEvents

                'open in browser, if OK & desired
                mShowWarnings = True
                If (ExportDocument() = 0) Then
                    If gOptions.openInBrowser Then
                        openInBrowser gOptions
                    End If
                    'increment statistics
                    MTIncrementStatisticBy "CMP", 1
                    If gOptions.useMathML Then
                        Select Case gOptions.mathMLtarget
                        Case "XHTML+MathML"
                            MTIncrementStatisticBy "MPXHTML", 1
                        Case "MathPlayer (IE5.5+ behavior)"
                            MTIncrementStatisticBy "MPMP", 1
                        Case "Multi-browser (UMSS)"
                            MTIncrementStatisticBy "MPUMSS", 1
                        End Select
                    Else
                        MTIncrementStatisticBy "MPGIF", 1
                                                                If gOptions.ieOnly Then
                                     MTIncrementStatisticBy "MPGIFIE", 1
                                                                End If
                    End If
                End If

            End If
        End If

        'restore Word's Background Save setting
        'to avoid Word corrupting doc, if b/g saves were ON we re-save original doc (bug# 2690)
        Application.options.BackgroundSave = oldBS
        If oldBS Then
            With ActiveDocument
                If Len(.path) > 0 And Not .ReadOnly Then
                    'catch error locally in case drive full, locked etc.
                    'also disable any error dialogs Word may try to display
                    Application.DisplayAlerts = wdAlertsNone
                    On Error Resume Next
                    .saved = False
                    .Save
                    .saved = True
                    On Error GoTo 0
                    Application.DisplayAlerts = wdAlertsAll
                End If
            End With
        End If
    End If
End Sub

'Returns True if the MathPage WLL version is OK, else displays error & returns False
Public Function IsWLLVersionOK()
    Dim stat As Boolean
    Dim wllVer As Long
    Dim msg As String

    stat = False
    'get the API Version (loads DLL)
    wllVer = MPAPIVersion(MPAPI_VERSION)
    'check the version against our constants
    If (wllVer > mpversMajVerHi) Or (wllVer < mpversMajVerLo) Then
        msg = MTLib.GetUserString2("1655", "3255", "The version of this command doesn't match the version of the MathPage WLL. Reinstall MathType to fix this condition.")
    ElseIf (wllVer < mtversMinVer) Then
        msg = MTLib.GetUserString2("1656", "3256", "A more recent version of MathType's WLL is required to use this command. Reinstall MathType to fix this condition.")
    Else
        stat = True
    End If

    If Not stat Then
        MsgBox msg, vbCritical, MTLib.GetUserString2("1609", "3209", "MathType Commands for Microsoft Word Error")
    End If
    IsWLLVersionOK = stat
End Function

'Exports all .doc documents in selected folder
'This module is not yet exposed in the user interface, but the ExportAllMathPage stub is
'Users can attach a toolbar item to ExportAllMathPage, which is always loaded
Sub ExportAll()
    Dim batchOptions As DialogOptions

    If Not IsWLLVersionOK() Then
        Exit Sub
    End If

    MsgBox "First, open any document, and all .doc files in that directory will be processed.", _
        vbOKOnly, kMPName

    'put up Word's File Open dialog
    With Dialogs(wdDialogFileOpen)
        If .Show <> -1 Then
            Exit Sub
        End If
    End With

    MsgBox "Now set the MathPage settings to be applied to all documents. All output files will be placed in the same directory as the file you specify, using the same extension but substituting the filename."

    'must call Init() before bringing up MathPage dialog
    Init

    'put up the MathPage dialog (no need to translate, internal use only in current form)
    If Not MathPageDlg.ShowDialog() Then
        Exit Sub
    End If

    mShowWarnings = False 'don't show warning dialogs
    batchOptions = gOptions 'save the options away as gOptions gets overwritten
    ExportAllDocsInFolder ActiveDocument.path, batchOptions

End Sub

'get list of files in a directory, storing the filenames in an array and returning the count
Public Function GetDirAsArray(pathName As String, attributes As Long, filenames() As String) As Long
    Const kAllocSize As Long = 100
    Dim fileName As String
    Dim numFiles As Long

    ReDim filenames(kAllocSize) As String

    numFiles = 0
    fileName = dir(pathName, attributes)
    While fileName <> ""
        numFiles = numFiles + 1
        If numFiles > UBound(filenames) Then
            ReDim Preserve filenames(UBound(filenames) + kAllocSize) As String
        End If
        filenames(numFiles) = fileName
        fileName = dir 'read next
    Wend

    GetDirAsArray = numFiles
End Function

'Exports all .doc documents in given folder using current MathPage options (gOptions)
Sub ExportAllDocsInFolder(sourceDir As String, batchOptions As DialogOptions)
    Dim fileName As String
    Dim ext As String
    Dim Doc As Document
    Dim pos As Long
    Dim numFiles As Long
    Dim filenames() As String
    Dim dirPath As String
    Dim outputDir As String
    Dim attrib As Long
    Dim i As Long

    Application.ScreenRefresh
    outputDir = MTLib.GetParentDirFromPath(batchOptions.fileName)
    pos = InStrR(batchOptions.fileName, ".")
    ext = Strings.Mid$(batchOptions.fileName, pos + 1)

    'get all .doc filenames into an array
#If Win32 Then
    dirPath = sourceDir & Application.PathSeparator & "*.doc"
    attrib = vbNormal
#Else
    dirPath = sourceDir & Application.PathSeparator
    attrib = macID("W8BN")
#End If
    numFiles = GetDirAsArray(dirPath, attrib, filenames)

    'export each file using the batch options specified in the MathPage dialog
    For i = 1 To numFiles
        fileName = filenames(i)
#If Win32 Then
        Documents.Open fileName
#Else
        Documents.Open sourceDir & Application.PathSeparator & fileName
#End If
        Set Doc = ActiveDocument
        If DocumentOK(Doc) Then
            GetOptions gOptions
            gOptions.ieOnly = batchOptions.ieOnly
            gOptions.mathZoom = batchOptions.mathZoom
            gOptions.mathMLtarget = batchOptions.mathMLtarget
            gOptions.useMathML = batchOptions.useMathML
            gOptions.fileName = outputDir & Application.PathSeparator & MTLib.ReplaceExtension(fileName, ext)
            gOptions.antiAlias = batchOptions.antiAlias
            ExportDocument
            ActiveDocument.Close
        End If
    Next i

End Sub

'Various initialization stuff (globals etc.)
Sub Init()
    gAppVersion = Val(Application.version)

    If gAppVersion = kWord97 Then
        mFolderSuffix = "_Files"
    Else
        mFolderSuffix = GetFolderSuffix()
    End If

    'For Word97 we must make sure that the HTML converter is installed
    If gAppVersion = kWord97 Then
        If Not HTMLConverterFound Then
            MsgBox MTLib.GetUserString("!1001You must install the HTML converter before you can use this command."), _
                vbOKOnly + vbExclamation, kMPName
            Exit Sub
        End If
    Else
        gHTMLFormatID = GetW2000HTMLFormatID()
    End If
End Sub

'Returns True & sets mDocument if document is OK to be exported to HTML
'warns if doc hasn't ever been saved, asks what to do if doc is dirty
'If error, displays message & returns False.
Function DocumentOK(Doc As Document) As Boolean
    Dim stat As Long

    DocumentOK = False

    'If doc has never been saved, refuse to run
    If Len(Doc.path) = 0 Then
        MsgBox MTLib.GetUserString("!1002This document has not been saved. Please save it and then use this command again."), _
            vbOKOnly + vbExclamation, kMPName
        Exit Function
    End If

    'Can't save templates as HTML in Word on the Mac
#If Mac Then
    If Doc.Type = wdTypeTemplate Then
        MsgBox MTLib.GetUserString("!1033Can't save a template as HTML in this version of Word."), _
            vbOKOnly + vbExclamation, kMPName
        Exit Function
    End If
#End If

    'If it needs saving, error if read-only, ask otherwise
    If Not Doc.saved Then
        'handle modified read-only documents (treat as if never saved)
        If Doc.ReadOnly Then
            stat = MsgBox(MTLib.GetUserString("!1003This read-only document has been modified. Please save it or re-open it and then use this command again."), _
                vbOKOnly + vbExclamation, kMPName)
            Exit Function
        Else
            stat = MsgBox(MTLib.GetUserString("!1004This document has been modified since it was last saved. Do you want to save changes?"), _
                vbOKCancel + vbQuestion, kMPName)
        End If
        If stat = vbOK Then
            Doc.Save
        Else
            Exit Function
        End If
    End If

    Set mDocument = Doc
    DocumentOK = True
End Function

'Processes mDocument
'   turn off screen updating & warning dialogs
'   turn off Field Codes, smart cut & paste
'   save current document and selection
'   process equation numbers & references
'   process equations
'   call save as HTML command
'   close working doc, re-open current document and restore selection
'   restore other Word settings
'   turn on screen updating & alerts
'Returns 0 for success
Private Function ExportDocument() As Long
    Dim docFullName As String
    Dim baseName As String
    Dim parentDir As String
    Dim supportFileDir As String
    Dim i As Long
    Dim pos As Long
    Dim stat As Long
    Dim restored As Boolean     'True if app settings restored
    Dim updatingRestored As Boolean

    ExportDocument = 0
    restored = True
    updatingRestored = True

    System.Cursor = wdCursorWait
    docFullName = mDocument.fullName

    'must remove all files in _files subdirectory before making our GIFs...
    '...or create this directory if it doesn't already exist
    pos = InStrR(gOptions.fileName, ".")
    mExtension = Strings.Mid$(gOptions.fileName, pos + 1)
    baseName = Strings.left$(gOptions.fileName, pos - 1)
    supportFileDir = baseName & mFolderSuffix
    mTempGIFPath = supportFileDir & Application.PathSeparator & kTempGIF

    'cleanup old files, create directory if necessary
    stat = DeleteOldFiles(gOptions.fileName)
    If stat <> mpOK Then GoTo fileError

    'set up for master documents and subdocuments...
    If mDocument.IsMasterDocument Then
        With mDocument.Subdocuments
            If .Expanded = False Then
                If Not ExpandSubDocuments(mDocument.Subdocuments) Then
                    ExportDocument = -1 'failure
                    Exit Function
                End If
            End If
            'build array of subdoc names - we need this several times
            'can't iterate subDocs coll unless expanded - Word crashes!
            If .count <= 0 Then
                ReDim mSubDocNames(0)
            Else
                ReDim mSubDocNames(1 To .count)
                parentDir = MTLib.GetParentDirFromPath(gOptions.fileName)
                For i = 1 To .count
                    'Word always generates subdocs with ".htm" extension
                    mSubDocNames(i) = parentDir & Application.PathSeparator & _
                        MTLib.ReplaceExtension(MTLib.GetFileNameFromPath(.item(i).name), mExtension)
                    stat = DeleteOldFiles(mSubDocNames(i))
                    If stat <> mpOK Then Exit For
                Next
                If stat <> mpOK Then GoTo fileError
            End If
        End With

    'set up for frameset documents
    ElseIf mDocument.ActiveWindow.Panes.count > 1 Then
        'save away document names for each frame
        With mDocument.ActiveWindow.Panes
            ReDim mSubDocNames(1 To .count)
            For i = 1 To .count
                'Word always generates frames with ".htm" extension
                mSubDocNames(i) = MTLib.ReplaceExtension(.item(i).Document.fullName, mExtension)
                stat = DeleteOldFiles(mSubDocNames(i))
                If stat <> mpOK Then Exit For
            Next
            If stat <> mpOK Then GoTo fileError
        End With
    End If

    'save Word settings and set app the way we want it...
    MTLib.SetScreenUpdate False
    updatingRestored = False
    Application.DisplayAlerts = wdAlertsNone
    With Application.options
        mOldSmartCutPaste = .SmartCutPaste
        .SmartCutPaste = False
        mOldPagination = .Pagination
        .Pagination = True
        restored = False
    End With

    'save document settings we want to restore (selection, etc.)
    With mOldSelection
        .start = Selection.start
        .end = Selection.end
        .storyType = Selection.storyType
        .storyPos = FindStory(mDocument, .storyType, Selection.Range)
        .viewType = mDocument.ActiveWindow.View.Type
        mDocument.ActiveWindow.View.Type = wdNormalView
    End With

    'these can cause errors on locked docs
    On Error Resume Next
    mOldTrackRevisions = mDocument.TrackRevisions
    mDocument.TrackRevisions = False

    mOldShowFieldCodes = mDocument.ActiveWindow.View.ShowFieldCodes
    mDocument.ActiveWindow.View.ShowFieldCodes = False

    'initialize the WLL
    On Error GoTo miscError
    InitWLL MTLib.GetParentDirFromPath(baseName), MTLib.GetFileNameFromPath(supportFileDir)

    'copy supporting files before processing (can vary by target)
    stat = CopySupportFiles
    If stat <> mpOK Then
        MPDocTerm
        MsgBox MTLib.GetUserString("!1029An error occurred copying supporting files, a file may be missing. Please reinstall MathType and try again."), _
            vbOKOnly + vbExclamation, kMPName
        GoTo miscError
    End If

    InitProgressInfo
    'convert the doc to HTML
    MTProgress.Show 'calls ProcessDocument
    Set gProgressInfo.form = Nothing

    'restore app settings here so document doesn't show an Undo action
    '(resetting SmartCutPaste causes this)
    Application.options.SmartCutPaste = mOldSmartCutPaste
    Application.options.Pagination = mOldPagination
    restored = True

    'close modified doc & re-open & activate original
    ReOpenDocument docFullName

    'if OK, clean up the Word-generated HTML & rename main file
    If mResult = 0 Then
        PostProcessHTMLDocs mDocument, gOptions.fileName
        RenameHTMLDocs mDocument, gOptions.fileName
        Kill mTempGIFPath 'delete the temporary placeholder GIF
    Else
        ExportDocument = -1
    End If
    stat = MPDocTerm
    If stat <> mpOK Then GoTo fileError

    'restore original doc's settings & selection etc.
    mDocument.ActiveWindow.View.ShowFieldCodes = mOldShowFieldCodes
    mDocument.TrackRevisions = mOldTrackRevisions
    RestoreOriginalSelection mOldSelection

    'reset remaining App settings
    Application.DisplayAlerts = wdAlertsAll
    MTLib.SetScreenUpdate True
    updatingRestored = True

    If mShowWarnings Then
        If mHasFloatingEquations Then
            MsgBox MTLib.GetUserString("!1005Floating equations were detected in the document! These were ignored and left for Word to handle."), _
                vbOKOnly + vbExclamation, kMPName
        End If
        If mHasMissingFonts Then
            MsgBox MTLib.GetUserString("!1027This document contains characters from missing or mismatched fonts. " & _
                "This prevents MathPage from generating the correct HTML. These characters were converted to question marks in the MathPage document."), _
                vbOKOnly + vbExclamation, kMPName
        End If
        If mHasTranslatorErrors Then
            If gOptions.insertTranslationError Then
               MsgBox MTLib.GetUserString2("1031", "1035", "Translation errors occurred during processing. The following text has been inserted immediately after any symbols or equations that were not successfully translated: ") & _
                      MTLib.GetUserString2("1032", "1036", "***TRANSLATION ERROR***"), _
                   vbOKOnly + vbExclamation, kMPName
            Else
               MsgBox MTLib.GetUserString2("1031", "1037", "Translation errors occurred during processing."), _
                   vbOKOnly + vbExclamation, kMPName
            End If
        End If
    End If

    GoTo done

fileError:
    ShowFileErrorMessage stat

miscError:
    ExportDocument = -1
    If Not restored Then    'restore app settings in case error threw us here
        Application.options.SmartCutPaste = mOldSmartCutPaste
        Application.options.Pagination = mOldPagination
    End If
    If Not updatingRestored Then
        MTLib.SetScreenUpdate True
    End If

done:
    System.Cursor = wdCursorNormal
End Function

'expand subdocuments of master document
'trap error raised when user chooses 'No' to Word's 'expand w/o this subdoc?' dialog
Function ExpandSubDocuments(subDocs As Subdocuments) As Boolean
    On Error GoTo failure
    subDocs.Expanded = True
    ExpandSubDocuments = True
    Exit Function
failure:
    ExpandSubDocuments = False
    Exit Function
End Function

Sub ProcessDocument()
    'install error handler and process the document
    On Error GoTo processError

    mResult = 0
    mSymbolCount = 0
    mNormalText = GetNormalText(mDocument) 'do after screen-updating turned off
    mHasFloatingEquations = False
    mHasMissingFonts = False
    mHasTranslatorErrors = False

    ProcessAllPanes

    'create HTML file with correct name so Word uses correct support files dir
    ExportToHTML gOptions.fileName

    Exit Sub

processError:
    MsgBox MTLib.GetUserString("!1006An unexpected processing error occurred, please try again. If this continues to occur, try reinstalling MathType.") & vbCrLf & _
        "ProcessDocument error: " & err.Number & ", " & err.Description, vbCritical + vbOKOnly, kMPName
    mResult = -1
End Sub

Private Sub ProcessAllPanes()
    Dim numPanes, i As Long
    numPanes = mDocument.ActiveWindow.Panes.count
    If numPanes > 1 Then
        Dim fsDoc As Document
        Set fsDoc = mDocument
        For i = 1 To numPanes
            Set mDocument = fsDoc.ActiveWindow.Panes(i).Document
            ProcessSinglePane
        Next
        Set mDocument = fsDoc
    Else
        ProcessSinglePane
    End If
End Sub

Private Sub ProcessSinglePane()

    'process the document...

    'if user doesn't want to see revisions, just accept all changes in our copy of doc
    If Not mDocument.ShowRevisions Then
        mDocument.Revisions.AcceptAll
    End If

    ProcessSymbols
    ProcessSymbolFields
    ProcessEquations
    ProcessEqnNumbers mExtension

End Sub

Private Sub InitProgressInfo()
    With gProgressInfo
        .title = MTLib.GetUserString("!0900Publish to MathPage")
        .activeString = 0
        .numStrings = 4
        .default(kPROGRESS_SYMBOL) = MTLib.GetUserString("!1017Processing Symbols...")
        .default(kPROGRESS_EQUATION) = MTLib.GetUserString("!1025Processing Equations...")
        .default(kPROGRESS_EQNNUM) = _
            MTLib.GetUserString("!1026Processing Equation Numbers and References...")
        .default(kPROGRESS_HTML) = MTLib.GetUserString("!1016Processing HTML")
        .default(5) = ""
        .default(6) = ""
        .procID = kPROGRESS_MATHPAGEPROC
    End With
End Sub

'Inits WLL, throws error
Private Sub InitWLL(docDir As String, supportDir As String)
    Dim wllFLags As Integer
    Dim stat As Long

    wllFLags = 0
    If gOptions.ieOnly = False Then wllFLags = wllFLags Or mpdFullCompatibility
    If gOptions.useMathML Then
        wllFLags = wllFLags Or mpdGenMathML
    ElseIf gOptions.mathZoom Then
        wllFLags = wllFLags Or mpdMathZoom
    End If

    stat = MPDocInit(docDir, supportDir, gAppVersion, wllFLags, gOptions.mathMLtarget)
    If stat <> mpOK Then
        Dim msg As String
        If stat = mtFILE_NOT_FOUND Then
            msg = MTLib.GetUserString("!1030Translator file is missing. Please re-install MathType and try again.")
        Else
            msg = MTLib.GetUserString("!1006An unexpected processing error occurred, please try again. If this continues to occur, try reinstalling MathType.")
        End If
        MsgBox msg, vbCritical + vbOKOnly, kMPName
        err.Raise vbObjectError + kInitWLLError, "", MTLib.GetUserString("!1007Document initialization error: ") & stat
    End If
End Sub

Private Sub ShowFileErrorMessage(err As Long)
    Dim msg As String
    If err = mpFILE_INVALID Then
        msg = MTLib.GetUserString("!1008The filename is invalid, please check the whole filename and try again.")
    ElseIf err = mpFILE_NO_ACCESS Then
        msg = MTLib.GetUserString("!1009Access to the file or a supporting file was denied, please check your permissions and try again.")
    Else
        msg = MTLib.GetUserString("!1010A disk error occurred, please check your drive and try again.")
    End If
    MsgBox msg, vbCritical + vbOKOnly, kMPName
End Sub

'close modified doc & re-open & activate original
Private Sub ReOpenDocument(ByRef fullName As String)
    If gAppVersion = kWord97 Then
        ReOpenDocument97 (fullName)
    Else
        ReOpenDocument2000 (fullName)
    End If
End Sub

'close modified doc & re-open & activate original for Word 97
Private Sub ReOpenDocument97(ByRef fullName As String)
    Dim htmlDoc As Document
    Dim oldLeft As Long, oldTop As Long, oldHeight As Long, oldWidth As Long, oldState As Long

    On Error Resume Next
    Set htmlDoc = mDocument
    If htmlDoc.fullName = fullName Then
        'must not have done save as so re-open discarding changes
        Set mDocument = Documents.Open(fileName:=fullName, Revert:=True)
    Else
        With Windows(htmlDoc.name)
            oldState = .WindowState
            oldLeft = .left
            oldTop = .top
            oldHeight = .height
            oldWidth = .width
        End With
        Set mDocument = Documents.Open(fileName:=fullName)
        With mDocument.ActiveWindow
            If oldState = wdWindowStateMaximize Then
                .WindowState = oldState
            Else
                .left = oldLeft
                .top = oldTop
                .width = oldWidth
                .height = oldHeight
            End If
        End With
        htmlDoc.Close wdDoNotSaveChanges    'discard changes just in case...
    End If
    mDocument.Activate
End Sub

'close modified doc & re-open & activate original for Word 2000
Private Sub ReOpenDocument2000(ByRef fullName As String)
    Dim htmlDoc As Document
    Dim oldLeft As Long, oldTop As Long, oldHeight As Long, oldWidth As Long, oldState As Long

    On Error Resume Next
    Set htmlDoc = mDocument
    If htmlDoc.fullName = fullName Then
        'must not have done save as so re-open discarding changes
        Set mDocument = Documents.Open(fileName:=fullName, Revert:=True)
    Else
        Dim i As Long
        Dim ourWin As Window
        For i = 1 To Windows.count
            If Windows(i).Document = htmlDoc Then
                Set ourWin = Windows(i)
                Exit For
            End If
        Next

        If Not ourWin Is Nothing Then
            With ourWin
                oldState = .WindowState
                oldLeft = .left
                oldTop = .top
                oldHeight = .height
                oldWidth = .width
            End With
        End If
        'this scheme does not work on Office XP when document contains frames
#If Win32 Then
        If gAppVersion >= kWordX Then
            Set mDocument = Documents.Open(fileName:=fullName, visible:=True)
        Else
            Set mDocument = Documents.Open(fileName:=fullName, visible:=False)
            If mDocument.Windows.count = 0 Then
                Set mDocument = Documents.Open(fileName:=fullName, visible:=True)
            End If
        End If
#Else
        If gAppVersion >= kWordX Then
            Set mDocument = Documents.Open(fileName:=fullName)
        Else
            Set mDocument = Documents.Open(fileName:=fullName)
            If mDocument.Windows.count = 0 Then
                Set mDocument = Documents.Open(fileName:=fullName)
            End If
        End If
#End If
        With mDocument.ActiveWindow
            If oldState = wdWindowStateMaximize Then
                .WindowState = oldState
            Else
                .left = oldLeft
                .top = oldTop
                .width = oldWidth
                .height = oldHeight
            End If
#If Win32 Then
            .visible = True
#End If
        End With
        htmlDoc.Close wdDoNotSaveChanges    'discard changes just in case...
    End If
    mDocument.Activate
End Sub

'Run HTML docs thru WLL
Private Sub PostProcessHTMLDocs(Doc As Document, ByRef fileName As String)
    Dim i As Long
    Dim oldname As String

    PostProcessDoc fileName, Doc.IsMasterDocument
    If Doc.IsMasterDocument Or (Doc.ActiveWindow.Panes.count > 1) Then
        For i = 1 To UBound(mSubDocNames)
            oldname = MTLib.ReplaceExtension(mSubDocNames(i), kDefaultHTMLExtension)
            If oldname <> mSubDocNames(i) Then
                Name oldname As mSubDocNames(i)
            End If
            PostProcessDoc mSubDocNames(i), False
        Next
    End If
End Sub

'Process xxx.htm into xxx.htm.tmp, throws if error
Private Sub PostProcessDoc(ByRef fileName As String, isMasterDoc As Boolean)
    Dim tempFile As String
    Dim stat As Long

    tempFile = fileName & ".tmp"
    stat = MPProcessHTML2(fileName, tempFile, isMasterDoc)
    If stat <> mpOK Then
        MsgBox MTLib.GetUserString("!1006An unexpected processing error occurred, please try again. If this continues to occur, try reinstalling MathType.") & vbNewLine & _
            MTLib.GetUserString("!1012Processing HTML error: ") & stat, vbCritical + vbOKOnly, kMPName
        err.Raise vbObjectError + kPostProcessDocError, "", MTLib.GetUserString("!1012Processing HTML error: ") & stat
    End If
End Sub

'Run HTML doc thru WLL. If master doc, handles subdocs too.
Private Sub RenameHTMLDocs(Doc As Document, ByRef mainFileName As String)
    Dim i As Long

    RenameHTMLDoc mainFileName, mainFileName & ".tmp", mKeepWordHTMLFile
    If Doc.IsMasterDocument Or (Doc.ActiveWindow.Panes.count > 1) Then
        For i = 1 To UBound(mSubDocNames)
            RenameHTMLDoc mSubDocNames(i), mSubDocNames(i) & ".tmp", mKeepWordHTMLFile
        Next
    End If
End Sub

'Renames tempfile to fileName, optionally keeping fileName around with a (Word)
Private Sub RenameHTMLDoc(ByRef fileName As String, ByRef tempFile As String, keepWord As Boolean)
    On Error Resume Next
    Kill fileName & "(Word).htm"   'must do else rename fails
    If keepWord Then
        Name fileName As (fileName & "(Word).htm")
    Else
        Kill fileName
    End If
    Name tempFile As fileName
End Sub

'Deletes existing file and deletes & re-creates its supporting folder too
Private Function DeleteOldFiles(ByRef fileName As String) As Long
    DeleteOldFiles = MPFileCleanup(fileName, mFolderSuffix)
End Function

'copies supporting files to the appropriate places (.js -> doc. dir, empty.gif -> supporting files dir)
'but can be customized in registry
'also copy empty.gif to temp.gif for use as dummy picture placeholder for eqns & symbols
Private Function CopySupportFiles() As Long
    Dim MPDir As String
    CopySupportFiles = MPCopySupportFiles()
    If CopySupportFiles = mpOK Then
        MPDir = MTLib.GetMathTypeDir & Application.PathSeparator & kMPName
        On Error GoTo copyerror:
        FileCopy MPDir & Application.PathSeparator & kEmptyGIF, mTempGIFPath
    End If
    Exit Function
copyerror:
    CopySupportFiles = mtFILE_NOT_FOUND
End Function

'Restores original selection
Private Sub RestoreOriginalSelection(oldSelection As SelInfo)
    Dim i As Long
    Dim newSel As Range

    With oldSelection
        mDocument.ActiveWindow.View.Type = .viewType
        If .storyPos >= 0 Then
            Set newSel = mDocument.StoryRanges(.storyType)
            i = 0
            While i < .storyPos
                Set newSel = newSel.NextStoryRange
                i = i + 1
            Wend
            newSel.SetRange .start, .end
            newSel.Select
        End If
    End With
End Sub

'Opens document in default browser
Private Sub openInBrowser(ByRef options As DialogOptions)
    MPOpenFileInBrowser options.fileName
End Sub

'Reads 'options' from doc. properties etc., sets globals
Private Sub GetOptions(ByRef options As DialogOptions)
    Dim path As String

    'non-existent properties throw an excception...
    On Error Resume Next

    'read reg. to see if we should always keep (Word) file
    mKeepWordHTMLFile = GetBoolOption(kMP_KeepWordHTMLFile, False)

    'get destination, default to doc name with .htm extension
    options.fileName = ""
    options.fileName = mDocument.CustomDocumentProperties(kMP_HTMLDest)
    If options.fileName = "" Then
        If (Strings.right$(mDocument.name, 5) = ".docx" Or _
            Strings.right$(mDocument.name, 4) = ".doc" Or _
            Strings.right$(mDocument.name, 4) = ".rtf" Or _
            Strings.right$(mDocument.name, 4) = ".xml") Then
                options.fileName = MTLib.ReplaceExtension(mDocument.fullName, kDefaultHTMLExtension)
        End If
    End If

    'get title
    options.title = MTLib.GetDocTitle(mDocument)

    'get MathML setting, default to off
    options.useMathML = GetBoolOption(kMP_UseMathML, False)
    options.mathMLtarget = GetStringOption(kMP_MathMLTarget, "")

    'get compatibility setting, default to ieOnly on Windows and All browsers on Mac
#If Win32 Then
    options.ieOnly = GetBoolOption(kMP_IEOnly, True)
#Else
    options.ieOnly = GetBoolOption(kMP_IEOnly, False)
#End If

    'get "Display in Browser" setting, default to True
    options.openInBrowser = GetBoolOption(kMP_OpenInBrowser, True)

    'get MathZoom setting, default to True
    options.mathZoom = GetBoolOption(kMP_MathZoom, True)

    'get TranslationError setting, default to on (insert after equation)
    options.insertTranslationError = (GetStringOption(kMP_TranslationError, "0") = "0")
End Sub

'Reads Boolean option from property, if missing then from registry, if missing then use default
Private Function GetBoolOption(ByRef name As String, default As Boolean) As Boolean
    Dim regValue As String

    On Error GoTo missing
    GetBoolOption = mDocument.CustomDocumentProperties(name)
    Exit Function

missing:
    regValue = GetPreference(HKEY_CURRENT_USER, mtreg_MT_MATHPAGE_KEY, name)
    Select Case regValue
    Case "1"
        GetBoolOption = True
    Case "0"
        GetBoolOption = False
    Case Else
        GetBoolOption = default
    End Select
End Function

'Reads String option from property, if missing then from registry, if missing then use default
Private Function GetStringOption(ByRef name As String, ByRef default As String) As String

    On Error GoTo missing
    GetStringOption = mDocument.CustomDocumentProperties(name)
    Exit Function

missing:
    GetStringOption = GetPreference(HKEY_CURRENT_USER, mtreg_MT_MATHPAGE_KEY, name)
    If GetStringOption = "" Then
        GetStringOption = default
    End If
End Function

'Unwraps and tags all eqn numbers and references...
Private Sub ProcessEqnNumbers(extension As String)
    Dim aStory As Range
    Dim aRange As Range
    Dim aField As Field
    Dim startPos As Long
    Dim endPos As Long
    Dim refStr As String
    Dim i As Long
    Dim oldHidden As Boolean

    MTLib.SetActiveProgressString kPROGRESS_EQNNUM

    'init bookmark array counter
    mNumBookmarks = 0

    'must handle case where style doesn't exist
    oldHidden = False
    If styleExists(mDocument, mtstyle_EQUATION_SECTION) Then
        oldHidden = mDocument.Styles(mtstyle_EQUATION_SECTION).font.Hidden
        mDocument.Styles(mtstyle_EQUATION_SECTION).font.Hidden = False
    End If

    BuildDuplicateBookmarksArray
    i = 1
    For Each aStory In mDocument.StoryRanges
        'We skip floating objects & header/footers (cause problems in W97)...
        If IsValidStory(aStory) Then
            'Some stories (textboxes) actually contain a range of ranges...
            'so we need an extra loop
            Do
                Set aRange = aStory.Duplicate
                For Each aField In aRange.Fields
                    MTLib.ShowProgressString _
                        MTLib.GetUserString("!1014Processing Equation Numbers (") & i & _
                        MTLib.GetUserString("!1015)")

                    If aField.Type = wdFieldMacroButton Then
                        If ProcessEqnNumField(aField) Then
                            i = i + 1
                        ElseIf gAppVersion = kWord97 Then
                            'remove our section break's hidden text that Word 97 displays
                            If InStr(1, Strings.LCase$(aField.Code.Text), Strings.LCase$(mtmacro_EDIT_EQUATION_SECTION), vbBinaryCompare) > 0 Then
                                aField.Select
                                aField.ShowCodes = True
                                InitFind Selection.find
                                With Selection.find
                                    .style = mtstyle_EQUATION_SECTION
                                    If .Execute Then
                                        Selection.delete
                                    End If
                                End With
                                aField.ShowCodes = False
                            End If
                        End If
                    ElseIf aField.Type = wdFieldGoToButton Then
                        If ProcessEqnRefField(aField, extension) Then
                            i = i + 1
                        End If
                    End If
                Next

                'if this story has a valid NextRange, process it (textboxes)
                Set aStory = aStory.NextStoryRange
            Loop While (IsObjectValid(aStory))
        End If
    Next 'story

    RemoveDuplicateBookmarks

    'restore original value if necessary
    If oldHidden Then
        mDocument.Styles(mtstyle_EQUATION_SECTION).font.Hidden = oldHidden
    End If
End Sub

'Build array of duplicate bookmarks and their new name
'For each bm, if one of ours & not already marked as a dup,
'check all other bookmarks for dups & add them to the list
Private Sub BuildDuplicateBookmarksArray()
    Dim bm As Bookmark, bm2 As Bookmark

    For Each bm In mDocument.Bookmarks
        If (InStr(1, Strings.LCase$(bm.name), "zeqnnum", vbBinaryCompare) > 0) And _
            (Not IsDuplicateBookmark(bm)) Then
                        ' "ZEqnNum"
            For Each bm2 In mDocument.Bookmarks
                If bm2 <> bm And bm2.Range.IsEqual(bm.Range) Then
                    ReDim Preserve mOldBookmark(mNumBookmarks + 1)
                    ReDim Preserve mNewBookmark(mNumBookmarks + 1)
                    mOldBookmark(mNumBookmarks) = bm2.name
                    mNewBookmark(mNumBookmarks) = bm.name
                    mNumBookmarks = mNumBookmarks + 1
                End If
            Next
        End If
    Next
End Sub

'Returns True if bm exists in duplicate bookmark list
Private Function IsDuplicateBookmark(bm As Bookmark) As Boolean
    Dim i As Long
    IsDuplicateBookmark = False
    Do While i < mNumBookmarks
        If mOldBookmark(i) = bm.name Then
            IsDuplicateBookmark = True
            Exit Do
        Else
            i = i + 1
        End If
    Loop
End Function
'Delete all bookmarks in duplicate list
Private Sub RemoveDuplicateBookmarks()
    Dim i As Long
    Do While i < mNumBookmarks
        mDocument.Bookmarks.item(mOldBookmark(i)).delete
        i = i + 1
    Loop
End Sub

' Removes GotoButton field wrapper from an eqn ref & replaces with a hyperlink.
' Uses file's extension for master document. Returns True if successful
Private Function ProcessEqnRefField( _
    aField As Field, _
    extension As String) As Boolean

    Dim startPos As Long
    Dim endPos As Long
    Dim fieldCode As String
    Dim reference As String
    Dim refField As Field
    Dim newField As Field
    Dim address As String
    Dim i As Long

    ProcessEqnRefField = False

    'extract bookmark name (the reference) from field
    fieldCode = aField.Code.Text
    startPos = InStr(1, Strings.LCase$(fieldCode), "zeqnnum", vbBinaryCompare)  ' "ZEqnNum"
    If startPos > 0 Then
        endPos = InStr(startPos, fieldCode, " ", vbBinaryCompare)
        If endPos > 0 Then
            reference = Strings.Mid$(fieldCode, startPos, endPos - startPos)
        End If
    End If

    'exit if we didn't find what we were looking for
    If reference = "" Then
        Exit Function
    End If

    'search global array of duplicate bookmarks to see if we should be changing this one
    i = 0
    Do While i < mNumBookmarks
        If reference = mOldBookmark(i) Then
            reference = mNewBookmark(i)
            Exit Do
        Else
            i = i + 1
        End If
    Loop

    aField.Select
    aField.ShowCodes = True

    With Selection
        Set refField = .NextField
        If Not (refField Is Nothing) Then
            If .Fields(1).Type = wdFieldRef Then
                'delete wrapper field and contents, then add new Ref field
                'that points to same reference
                aField.delete
                Set newField = mDocument.Fields.Add(Range:=.Range, _
                    Type:=wdFieldRef, Text:=reference)
                newField.Select
                address = GetBookmarkAddress(reference, extension)
                mDocument.Hyperlinks.Add Anchor:=.Range, address:=address, _
                    SubAddress:=reference
                ProcessEqnRefField = True
            End If
        End If
    End With
End Function

'Returns correct file for reference, needed in master docs
'Returns "" if not master doc
Private Function GetBookmarkAddress(ByRef bookmarkName As String, ByRef extension As String) As String
    Dim found As Boolean

    'in case bookmark doesn't exist
    GetBookmarkAddress = ""
    If mDocument.IsMasterDocument Then
        Dim bm As Bookmark
        Dim Doc As Subdocument
        On Error GoTo abort
        found = False
        Set bm = mDocument.Bookmarks(bookmarkName)
        For Each Doc In mDocument.Subdocuments
            If bm.Range.InRange(Doc.Range) Then
                'Word's subdoc files always get an .htm extension
                GetBookmarkAddress = MTLib.ReplaceExtension(MTLib.GetFileNameFromPath(Doc.name), mExtension)
                found = True
                Exit For
            End If
        Next
        'if not found must be in master doc, so use its name
        If Not found Then
            GetBookmarkAddress = MTLib.ReplaceExtension(mDocument.name, extension)
        End If
abort:
    End If
End Function

' Removes MacroButton field wrapper from an eqn number, leaving inner field selected
' Note: can't use ranges as they don't expand over Fields like a Selection does
' Haven't found a more elegant way to remove the wrapper, unlike EqnRefs we have
' several fields and text inside the outer field.
' Returns True if successful
Private Function ProcessEqnNumField( _
    aField As Field) As Boolean
    Dim i As Long
    Dim bm As Bookmark
    Dim newName As String
    Dim found As Boolean

    DoEvents
    ProcessEqnNumField = False

    If InStr(1, Strings.LCase$(aField.Code.Text), "mtplaceref", vbBinaryCompare) = 0 Then ' "MTPlaceRef"
        Exit Function
    End If

    aField.Select
    aField.ShowCodes = True

    'select the part we want to keep
    Selection.MoveRight wdCharacter, 1
    Selection.moveLeft wdCharacter, 1
    i = 0
    Do Until (InStr(Selection.Text, "MTEqn " + Strings.ChrW(&H5C) + "h") > 0)
        Selection.moveLeft wdCharacter, 1, wdExtend
        i = i + 1
        If i > 30 Then
            Exit Function 'abort if we appear to have gone too far
        End If
    Loop

    'remove and save inner data, then delete outer field & insert insides again
    Selection.Cut
    Selection.MoveRight wdCharacter, 1
    Selection.moveLeft wdCharacter, 1, wdExtend
    Selection.delete wdCharacter, 1

    'paste w/Range so pasted data is part of the range (doesn't work for Selection!)
    Selection.Range.Paste

    ProcessEqnNumField = True
End Function

'Wrapper around Word's SaveAsHTML command
'Gets destination from Custom Doc property HTMLDest
Private Sub ExportToHTML(ByRef destFile As String)
    MTLib.SetActiveProgressString kPROGRESS_HTML

    If gAppVersion = kWord97 Then
        ExportToHTML97 destFile
    Else
        ExportToHTML2000 destFile
    End If
End Sub

Private Sub ExportToHTML97(ByRef destFile As String)
    mDocument.CustomDocumentProperties.Add _
        name:="DocumentEncoding", Type:=msoPropertyTypeString, value:="UTF-8", LinkToContent:=False

    mDocument.SaveAs fileName:=destFile, FileFormat:=gHTMLFormatID, _
        LockComments:=False, Password:="", AddToRecentFiles:=False, _
        WritePassword:="", ReadOnlyRecommended:=False, EmbedTrueTypeFonts:=False, _
        SaveNativePictureFormat:=False, SaveFormsData:=False, SaveAsAOCELetter:=False
    Set mDocument = ActiveDocument
End Sub

Private Sub ExportToHTML2000(ByRef destFile As String)
    Dim oldUpdateLinksOnSave As Boolean
    Dim oldCheckIfOfficeIsHTMLEditor As Boolean
    Dim oldCheckIfWordIsDefaultHTMLEditor As Boolean
    Dim oldAlwaysSaveInDefaultEncoding As Boolean
    Dim oldEncoding As Long
    Dim oldSaveAsArchive As Boolean
    Dim i As Long
    Dim hyperLinkBase As String
    Dim suppFilesFolder As String
    Dim tempFolderName As String
    Dim fileList() As String
    Dim numFiles As Long

    'save current settings that we need to restore
    With Application.DefaultWebOptions
        oldUpdateLinksOnSave = .UpdateLinksOnSave
        oldAlwaysSaveInDefaultEncoding = .AlwaysSaveInDefaultEncoding
        oldEncoding = .Encoding
        .UpdateLinksOnSave = True
        .AlwaysSaveInDefaultEncoding = False
        .Encoding = msoEncodingUTF8 'force subdocs to use encoding

        'read-only props on Mac
        #If Win32 Then
        oldCheckIfOfficeIsHTMLEditor = .CheckIfOfficeIsHTMLEditor
        oldCheckIfWordIsDefaultHTMLEditor = .CheckIfWordIsDefaultHTMLEditor
        .CheckIfOfficeIsHTMLEditor = False
        .CheckIfWordIsDefaultHTMLEditor = False
        If gAppVersion >= kWordX Then
            SetWordXDefaultWebOptions oldSaveAsArchive
        End If
        #End If
    End With

    'won't restore these as modified doc is discarded
    With mDocument.WebOptions
        'Turn on .RoundTripHTML or else Mac Word can output HTML
        '(extra </a> tags, etc.)
        #If Mac Then
        .RoundTripHTML = True
        #End If
        .Encoding = msoEncodingUTF8
        'read-only props on Mac
        #If Win32 Then
        .OrganizeInFolder = True
        .UseLongFileNames = True
        .OptimizeForBrowser = False
        .RelyOnCSS = True
        #End If
    End With

    'master documents and subdocuments processing
    If mDocument.IsMasterDocument Then
        #If Win32 Then
        If gAppVersion >= kWordX Then
            'on Word XP and later, enable dialogs and alerts & select "Yes to (A)ll"
    'this is necessary to correctly save all sub-documents as HTML
        Application.DisplayAlerts = wdAlertsAll
        Application.Activate
        SendKeys MTLib.GetUserString("!1028+{TAB}~")
    End If
        #ElseIf Mac Then
            'on Mac 2001 and later, sendkeys is not available,
            'and docs are not saved with .htm extension,
            'so save each subdocument separately
            'this works very well and may be better for windows also
            'Dim subdoc As Subdocument
            Dim subDocs As Subdocuments
            Dim Doc As Document
            'must set the view back to master doc view
            'or else error when trying to open subdocs
            mDocument.ActiveWindow.View.Type = wdMasterView
            Set subDocs = ActiveDocument.Subdocuments
            For i = 1 To subDocs.count
                'Set subdoc = subdocs(i)
                'Set doc = subdoc.Open()
                Set Doc = subDocs(i).Open
                Doc.SaveAs fileName:=mSubDocNames(i), FileFormat:=wdFormatHTML, _
                    LockComments:=False, Password:="", AddToRecentFiles:=True, WritePassword _
                    :="", ReadOnlyRecommended:=False, EmbedTrueTypeFonts:=False, _
                    SaveNativePictureFormat:=False, SaveFormsData:=False, SaveAsAOCELetter:= _
                    False, HTMLDisplayOnlyOutput:=False
                Doc.Close
            Next i
        #End If
    End If

    'on the mac, to avoid Word's dialog about overwriting the supporting
    'files folder, rename the folder first since we've already created it,
    'then copy contents back later
    #If Mac Then
        Dim baseName As String
        Dim dotPos As Long
        baseName = destFile
        dotPos = InStrR(baseName, ".")
        If dotPos > 0 Then
            baseName = Strings.left$(baseName, dotPos - 1)
        End If
        suppFilesFolder = baseName & mFolderSuffix
        tempFolderName = baseName & "_MP"

        'first, delete the temp folder in case it already exists
        MTLib.DeleteDirectory (tempFolderName)

        'rename the supp files folder to our temp folder name
        On Error Resume Next 'workaround for word 2004
        Name suppFilesFolder As tempFolderName
        On Error GoTo 0
    #End If

    'FIX can get error 6048 here if there's another doc with same basename in this dir
    'they'll be sharing the same _Files directory, hence the error
    'we'll have already deleted this dir, so we've probably screwed things up! fix earlier
    mDocument.SaveAs fileName:=destFile, FileFormat:=gHTMLFormatID, _
        LockComments:=False, Password:="", AddToRecentFiles:=False, _
        WritePassword:="", ReadOnlyRecommended:=False, EmbedTrueTypeFonts:=False, _
        SaveNativePictureFormat:=False, SaveFormsData:=False, SaveAsAOCELetter:=False

    'if Word created a supporting files folder, copy its files to our temp folder,
    'then move our folder back to the right name
    #If Mac Then
    'copy files out of supporting files folder to our temp folder
    numFiles = MTLib.GetDirAsArray(suppFilesFolder & Application.PathSeparator, vbNormal, fileList)
    For i = 1 To numFiles
        On Error Resume Next
        FileCopy suppFilesFolder & Application.PathSeparator & fileList(i), _
            tempFolderName & Application.PathSeparator & fileList(i)
    Next i

    'delete the supporting files folder
    MTLib.DeleteDirectory (suppFilesFolder)

    'rename our temp folder back to the correct name
    On Error Resume Next 'workaround for mac word 2004
    Name tempFolderName As suppFilesFolder
    On Error GoTo 0
    #Else
    'if master document in XP or later, turn dialogs & alerts back off
    If gAppVersion >= kWordX And mDocument.IsMasterDocument Then
        Application.DisplayAlerts = wdAlertsNone
    End If
    #End If

    Set mDocument = ActiveDocument.ActiveWindow.Document

    'restore web options (have to set individually)
    With Application.DefaultWebOptions
        .UpdateLinksOnSave = oldUpdateLinksOnSave
        .AlwaysSaveInDefaultEncoding = oldAlwaysSaveInDefaultEncoding
        .Encoding = oldEncoding
        'only need to restore on Windows
        #If Win32 Then
        .CheckIfOfficeIsHTMLEditor = oldCheckIfOfficeIsHTMLEditor
        .CheckIfWordIsDefaultHTMLEditor = oldCheckIfWordIsDefaultHTMLEditor
        If gAppVersion >= kWordX Then
            RestoreWordXDefaultWebOptions oldSaveAsArchive
        End If
        #End If
    End With
End Sub

'Word 10-specific properties
Private Sub SetWordXDefaultWebOptions(ByRef oldSaveAsArchive As Boolean)
#If Win32 Then
    Dim defaultWebOpts As Object
    Set defaultWebOpts = Application.DefaultWebOptions
    With defaultWebOpts
        oldSaveAsArchive = .SaveNewWebPagesAsWebArchives
        .SaveNewWebPagesAsWebArchives = False
    End With
#End If
End Sub

Private Sub RestoreWordXDefaultWebOptions(oldSaveAsArchive As Boolean)
#If Win32 Then
    Dim defaultWebOpts As Object
    Set defaultWebOpts = Application.DefaultWebOptions
    defaultWebOpts.SaveNewWebPagesAsWebArchives = oldSaveAsArchive
#End If
End Sub

'Processes all symbols in the document.
'Search thru all text runs in the document, passing them to MPFindSymbol.
'It returns on the first 'interesting' character, which may be one we want to
'change into a character entity, or a GIF. It also returns on a '(', which is the
'placeholder Word uses to represent some symbols inserted via the InsertSymbol
'command. In this case we have to test the character using the InsertSymbol dialog
'to see if its a symbol we should turn into a GIF.
Private Function ProcessSymbols()
    Dim aStory As Range
    Dim start As Long
    Dim aRange As Range
    Dim aTable As Table

    MTLib.SetActiveProgressString kPROGRESS_SYMBOL

    For Each aStory In mDocument.StoryRanges
        'We skip floating objects & header/footers (cause problems in W97)...
        If IsValidStory(aStory) Then
            'Some stories (textboxes) actually contain a range of ranges...
            'so we need an extra loop to handle them
            Do
                aStory.Characters(1).Select
                Do While (Selection.Range.end < aStory.end - 1)
                    'select run of text in consistent font & size, cant do w/range
                    start = Selection.start
                    Selection.SelectCurrentFont

                    'If selection contains a table, first select up to the table
                    If SelectionInTable(aTable) Then
                        ' Two versions (for VBA) as Word97 doesn't allow nested tables
                        If gAppVersion = kWord97 Then
                            ProcessSymbolsInTable97 aTable
                        Else
                            ProcessSymbolsInTable aTable
                        End If
                    Else
                        'process first symbol in selection
                        'use range so we can control selection when fields involved
                        Set aRange = Selection.Range.Duplicate
                        aRange.start = start
                        ForceRangeFont aRange
                        FindSymbol aRange
                        Set aRange = Nothing
                    End If

                    'processed text is selected, so collapse to advance
                    Selection.Collapse wdCollapseEnd
                    DoEvents    'let Windows update if necessary
                Loop

                'if this story has a valid NextRange, process it (textboxes)
                Set aStory = aStory.NextStoryRange
            Loop While (IsObjectValid(aStory))
        End If
    Next 'story
End Function

'Returns True if start of the current selection is in a table, and returns outermost table
'Otherwise returns False and, if selection contains a table, moves selection's end to start of table
Private Function SelectionInTable(aTable As Table) As Boolean
    Dim firstTableStart As Long
    SelectionInTable = False
    If Selection.Range.Tables.count > 0 Then
        firstTableStart = Selection.Range.Tables(1).Range.start
        If Selection.start >= firstTableStart Then
            Set aTable = Selection.Range.Tables(1)
            SelectionInTable = True
        End If
        Selection.end = firstTableStart
    End If
End Function

'As SelectCurrentFont has problems with Tables (selection expands backwards among other things)
'we process tables a cell at a time. we also have to deal with nested tables.
'Leaves table selected when done.
'Returns True if successful
Private Function ProcessSymbolsInTable(aTable As Table) As Boolean
    Dim tableCell As Long
    Dim aCell As Cell, aRange As Range, aNestedTable As Table
    Dim cellStart As Long
    Dim selStart As Long, selEnd As Long
    Dim tableIndex As Long  'used to keep track of nested tables

    ProcessSymbolsInTable = False

    'loop thru table sequentially by cell
    '(rows & cols can have 'split' or 'merged' cells, so one
    ' can't loop thru table by row and then by col; i.e. tables may be non-square)
    For tableCell = 1 To aTable.Range.Cells.count

        Set aCell = aTable.Range.Cells(tableCell)
        cellStart = aCell.Range.start

        'if non-empty cell...
        If cellStart < aCell.Range.end Then

            'set up for processing current cell
            tableIndex = 1
            Selection.SetRange cellStart, cellStart
            selStart = Selection.start              ' remember selection start
            selEnd = GetCellEnd(aCell, tableIndex)  ' remember selection end (includes nested tables)

            'process entire cell
            Do While Selection.Range.end < (aCell.Range.end - 1)

                'process each run of text or nested table
                If SelectionInCellTable(aCell, tableIndex) Then
                    'process nested table
                    Set aNestedTable = aCell.Tables(tableIndex)
                    ProcessSymbolsInTable aNestedTable
                    tableIndex = tableIndex + 1
                    ' processed data is still selected, so collapse to advance selection
                    Selection.Collapse wdCollapseEnd
                    selStart = Selection.start              ' remember start
                    selEnd = GetCellEnd(aCell, tableIndex)  ' remember end (nested table dependent)
                Else
                    'process run of text, which may contain one or more symbols
                    Do While (selStart < selEnd)
                        Selection.SelectCurrentFont
                        'confine selection to current cell up to start of any nested table
                        If (Selection.start < selStart) Or (Selection.end > selEnd) Then
                            'must do in this order else selection changes don't have any effect
                            aCell.Select
                            Selection.end = selEnd
                            Selection.start = selStart
                        End If

                        ' avoid endless loops caused by hyperlinks (or any other unforeseen reason)
                        If Selection.start < selStart Then
                           selStart = selStart + 1
                        Else
                            'find and process any symbols
                            Set aRange = Selection.Range.Duplicate
                            ForceRangeFont aRange
                            FindSymbol aRange

                            ' processed data is still selected, so collapse to advance selection
                            Selection.Collapse wdCollapseEnd
                            selStart = Selection.start              ' remember start
                            selEnd = GetCellEnd(aCell, tableIndex)  ' remember end (nested table dependent)
                        End If
                    Loop
                End If

            Loop 'process entire cell

        End If 'non-empty cell

    Next tableCell

    ' must return table selected so it can be stepped over
    aTable.Select

    ProcessSymbolsInTable = True
End Function

'Special version of ProcessSymbolsInTable for Word97 which doesn't allow nested tables
'and doesn't have all the properties that Word2000 does (that we use)
'See comments above. Returns True if successful
Private Function ProcessSymbolsInTable97(aTable As Table) As Boolean
    Dim i As Long, j As Long
    Dim aCell As Cell, aRange As Range, aNestedTable As Table
    Dim cellStart As Long, cellEnd As Long
    Dim selStart As Long

    ProcessSymbolsInTable97 = False

    'loop thru table by cell
    For i = 1 To aTable.Range.Cells.count
        Set aCell = aTable.Range.Cells(i)
        cellStart = aCell.Range.start
        cellEnd = aCell.Range.end
        If cellEnd > cellStart Then                 ' skip empty cells
            Selection.SetRange cellStart, cellStart
            selStart = Selection.start              ' remember start

            'process entire cell
            Do While (Selection.Range.end < cellEnd - 1)

                Selection.SelectCurrentFont
                If (Selection.start < selStart) Or (Selection.end > cellEnd) Then
                    'must do in this order else selection changes don't have any effect
                    aCell.Range.Select
                    Selection.SetRange selStart, cellEnd - 1
                End If

                Set aRange = Selection.Range.Duplicate
                ForceRangeFont aRange
                FindSymbol aRange

                ' processed data is still selected, so collapse to advance selection
                Selection.Collapse wdCollapseEnd
                selStart = Selection.start              ' remember start
            Loop
        End If
    Next

    'must return table selected so it can be stepped over
    aTable.Select

    ProcessSymbolsInTable97 = True
End Function

Private Function SelectionInCellTable(aCell As Cell, cellTableIndex As Long) As Boolean
    If aCell.Tables.count < cellTableIndex Then
        SelectionInCellTable = False
    Else
        SelectionInCellTable = Selection.InRange(aCell.Tables(cellTableIndex).Range)
    End If
End Function

'Returns current end position for cell, bounded to start of ith table in cell.
Private Function GetCellEnd(aCell As Cell, cellTableIndex As Long) As Long
    If aCell.Tables.count < cellTableIndex Then
        GetCellEnd = aCell.Range.end - 1
    Else
        GetCellEnd = aCell.Tables(cellTableIndex).Range.start
    End If
End Function

'Searches for and processes first symbol in range
'Return True if successful (if a symbol was found)
'Sets Selection (collapsed) immediately past processed symbol or end of range
Private Function FindSymbol(aRange As Range) As Boolean
    Dim stat As Long
    Dim info As SymbolInfo

    FindSymbol = False

    info.kind = kSIDefault
    stat = MPFindSymbol(aRange, aRange.font.name, info)
    If stat >= 0 Then
        stat = stat + 1     'stat is 0-based
        aRange.Characters(stat).Select

        'insert the ENTITY tag with values from the returned info
        If info.kind = kSIEntity Then
            InsertEntity info.charCode
            FindSymbol = True

        'we're going to generate a GIF, so pass returned info along
        ElseIf info.kind = kSIGIF Then
            If ProcessSymbol(Selection.font.name, info) Then
                FindSymbol = True
            End If

        'found a possible placeholder, so test it...
        ElseIf info.kind = kSIInsertSymbolPlaceholder Then
            If AnalyzeSymbol() Then
                FindSymbol = True
            End If

        'found a char that we want to substitute for, or simply delete
        'examples are optional hyphen (we sometimes delete) & non-breaking hyphen
        ElseIf info.kind = kSISubstituted Then
            If info.charCode = 0 Then
                Selection.delete
            Else
                If info.isUniCode Then
                    Selection.Text = Strings.ChrW(info.charCode)
                Else
                    Selection.Text = Strings.Chr(info.charCode)
                End If
            End If
            FindSymbol = True

        'if font missing, replace character with '?' and
        'display a message at the end of processing
        ElseIf info.kind = kSIMissingFont Then
            Selection.Text = "?"
            mHasMissingFonts = True
            FindSymbol = True

        'unknown symbol kind - error
        Else
            MsgBox MTLib.GetUserString("!1018Unknown symbol kind returned: ") & info.kind, _
                vbOKOnly, kMPName
        End If
        Selection.Collapse wdCollapseEnd

        If mSymbolCount > 0 Then
            MTLib.ShowProgressString _
                MTLib.GetUserString("!1019Processing Symbols (") & mSymbolCount & _
                MTLib.GetUserString("!1015)")
        End If
    Else
        'nothing found, move on
        Selection.SetRange aRange.end, aRange.end
    End If
End Function

'Sometimes SelectCurrentFont returns a null font name (info from newsgroup)
'So we reduce selection size by 1/2 and keep trying again
Private Function ForceRangeFont(r As Range) As Boolean
    Dim selCount As Long

    ForceRangeFont = False
    selCount = r.Characters.count
    Do While r.font.name = ""
        selCount = selCount / 2
        If selCount < 1 Then
            selCount = 1
            Exit Do
        End If
        r.Collapse wdCollapseStart
        r.MoveEnd wdCharacter, selCount
    Loop
    ForceRangeFont = True
End Function

'Returns True if story is 'valid' and should be processed
'Returns False textboxes, headers & footers
Private Function IsValidStory(aStory As Range) As Boolean
    With aStory
        IsValidStory = _
            (.storyType = wdMainTextStory) Or _
            (.storyType = wdFootnotesStory) Or _
            (.storyType = wdEndnotesStory) Or _
            (.storyType = wdFootnotesStory) Or _
            (.storyType = wdCommentsStory)
    End With
End Function

'Processes all symbol fields, replacing them with our SYMBOL tag
Private Sub ProcessSymbolFields()
    Dim aStory As Range
    Dim aRange As Range
    Dim aField As Field

    For Each aStory In mDocument.StoryRanges
        'Some stories (textboxes) actually contain a range of ranges...
        'so we need an extra loop
        'We skip floating objects & header/footers (cause problems in W97)...
        If IsValidStory(aStory) Then
            Do
                Set aRange = aStory.Duplicate
                For Each aField In aRange.Fields
                    If aField.Type = wdFieldSymbol Then
                        aField.Select
                        If AnalyzeSymbol() = False Then
                            'Word ignores Symbol fields when converting to HTML...
                            'so we'll fixup all fields we're not turning into GIFs
                            ReplaceSymbolField
                        End If
                        MTLib.ShowProgressString _
                            MTLib.GetUserString("!1019Processing Symbols (") & mSymbolCount & _
                            MTLib.GetUserString("!1015)")
                    End If
                Next

                'if this story has a valid NextRange, process it (textboxes)
                Set aStory = aStory.NextStoryRange
            Loop While (IsObjectValid(aStory))
        End If
    Next 'story
End Sub

'Replaces selection with an inserted character
'Used when converting Symbol fields to a real character
Private Sub ReplaceSymbolField()
    Dim dlg As Dialog
    Dim fontName As String
    Dim unicode As Boolean

    On Error GoTo err
    Set dlg = Dialogs(wdDialogInsertSymbol)
    On Error GoTo 0

    'if dlg returns "normal text" then use the Selection's font
    If dlg.font = mNormalText Then
        fontName = Selection.font.name
    Else
        fontName = dlg.font
    End If

    unicode = True
    If dlg.charNum < 0 Then
        dlg.charNum = dlg.charNum + 4096
        unicode = False
    End If

    'don't delete selection, this replaces selection & keeps color etc.
    Selection.InsertSymbol dlg.charNum, fontName, unicode
err:
End Sub

'Tests selected character, if determined to be a symbol we want to tag
'we get its font, char code, size etc. and replace it with our tag.
'Leaves original character selected, or tag selected if inserted.
'Returns True if tag was inserted.
Private Function AnalyzeSymbol() As Boolean
    Dim dlg As Dialog
    Dim symInfo As SymbolInfo
    Dim fontName As String
    Dim ReplaceSymbol As Boolean
    Dim stat As Integer

    AnalyzeSymbol = False
    symInfo.kind = kSIDefault

    'in case the InsertSymbol 'dialog' throws an error...
    On Error GoTo errHandler
    Set dlg = Dialogs(wdDialogInsertSymbol)
    On Error GoTo 0

    'MsgBox "SymFont = " & dlg.Font & ", SelFont = " & Selection.Font.Name & vbCrLf & _
        ", CharNum = " & charNum & ", Uni = " & dlg.Unicode _
        & ", hint = " & dlg.Hint & ", size = " & Selection.Font.Size
    'if dlg returns "normal text" then use the Selection's font
    If dlg.font = mNormalText Then
        fontName = Selection.font.name
    Else
        fontName = dlg.font
    End If
    ReplaceSymbol = True

    ' handle character appropriately, ignore possible placeholders...
    If MPAnalyzeSymbol(dlg.charNum, fontName, symInfo) <> kSIDefault Then

        'insert the ENTITY tag with values from the returned info
        If symInfo.kind = kSIEntity Then
            InsertEntity symInfo.charCode
            AnalyzeSymbol = True

        'we're going to generate a GIF, so pass returned info along
        ElseIf symInfo.kind = kSIGIF Then
            AnalyzeSymbol = ProcessSymbol(fontName, symInfo)

        'if font missing, replace character with '?' and
        'display a message at the end of processing
        ElseIf symInfo.kind = kSIMissingFont Then
            Selection.Text = "?"
            mHasMissingFonts = True
            AnalyzeSymbol = True
        End If
    End If

    'leave before error handler
    Exit Function

errHandler:
    ' Get 4605 when object selected has a symbolic font associated but the
    ' Object model won't allow the Insert Symbol dialog to be opened (e.g. an equation)
    ' msg left in for debugging purposes; dont bother to translate
    If err.Number <> 4605 Then
        MsgBox MTLib.GetUserString("!1006An unexpected processing error occurred, please try again. If this continues to occur, try reinstalling MathType.") & vbCrLf & _
            "AnalyzeSymbol: error " & err.Number & "," & err.Description, vbCritical + vbOKOnly, kMPName
    End If
End Function

'Inserts DS entity tag for charCode.
'We use a fake font so that the tag is human-readable in the raw HTML and so that Word97's
'symbolic font handling gets bypassed (it screws up in the UTF-8 encoding)
Private Sub InsertEntity(charCode As Integer)
    Dim entityCode As Long

    If charCode < 0 Then
        entityCode = 65536 + charCode
    Else
        entityCode = charCode
    End If

    mSymbolCount = mSymbolCount + 1
    InsertTag ktagEntityStart & entityCode & ktagEnd
End Sub

Private Sub InsertTag(ByRef tag As String)
#If Win32 Then
    If gAppVersion = kWord97 Then
        InsertTag97 tag
    Else
        InsertTag2000 tag
    End If
#Else
    InsertTag97 tag
#End If
End Sub

'Inserts tag, replacing any current selection. We maintain current font attributes
'except for the name, which we change to a fake font, MPTempFont.
Private Sub InsertTag97(ByRef tag As String)
    Dim myRange As Range
    Dim inSelection As Boolean
    With Selection
        inSelection = (.start <> .end)
        Set myRange = .Range.Duplicate
        .Collapse wdCollapseEnd
        .Text = tag
        .font.name = "MPTempFont"
    End With
    If inSelection Then
        myRange.delete
    End If
End Sub

'Inserts tag, replacing any current selection.
'We insert a dummy picture and put the tag in the AlternativeText attrubute
Private Sub InsertTag2000(ByRef tag As String)
    Dim myRange As Range
    Dim inSelection As Boolean
    Dim eqn, placeholder As InlineShape
    Dim width, height As Single

    With Selection
        inSelection = (.start <> .end)

        'if replacing an equation, save its width and height
        If inSelection And Selection.InlineShapes.count >= 1 Then
            width = Selection.InlineShapes(1).width - kEqnPadding
            height = Selection.InlineShapes(1).height - kEqnPadding
        Else
            width = Selection.font.size
            height = Selection.font.size
        End If

        Set myRange = .Range.Duplicate
        .Collapse wdCollapseEnd

        If gAppVersion = kWord2007 Then
            ' this is a fix for http://valor:8080/browse/MT-3063
            Set placeholder = Selection.InlineShapes.AddPicture(fileName:=mTempGIFPath, LinkToFile:=False, SaveWithDocument:=True)
        Else
            Set placeholder = Selection.InlineShapes.AddPicture(fileName:=mTempGIFPath, LinkToFile:=True, SaveWithDocument:=False)
        End If
        placeholder.width = width
        placeholder.height = height
        placeholder.AlternativeText = tag

    End With
    If inSelection Then
        myRange.delete
    End If
End Sub

'Processes selected character for which the symInfo structure has been partially filled out
'Adds size, style and color values to the structure
'Leaves original character selected, or tag selected if inserted.
'Returns True if tag was inserted.
Private Function ProcessSymbol(ByRef fontName As String, ByRef symInfo As SymbolInfo) As Boolean
    Dim stat As Long
    Dim attrs As String
    Dim symbolID As String
    Dim tag As String
    Dim inSubSuper As Long
    Dim aRange As Range
    Dim endOfPara As String

    ProcessSymbol = False
    inSubSuper = kNoSubSuper

    With Selection.font
        If .Subscript Then
            inSubSuper = kSubScript
        ElseIf .Superscript Then
            inSubSuper = kSuperScript
        End If

        If inSubSuper = kNoSubSuper Then
            symInfo.size = .size
        Else
            symInfo.size = .size * kSuperSubscriptRatio
        End If

        symInfo.style = kPlainText
        If .Bold Then
            symInfo.style = symInfo.style Or kBoldText
        End If
        If .Italic Then
            symInfo.style = symInfo.style Or kItalicText
        End If
    End With

    With Selection
        If gAppVersion = kWord97 Then
            symInfo.color = GetFontColor97(.font)
        Else
            symInfo.color = GetFontColor2000(.font)
        End If

        attrs = Strings.Space$(256)
        symbolID = Strings.Space$(256)
        stat = MPProcessSymbol("ch<id><res>.gif", fontName, symInfo, symbolID, attrs)
        If stat = mpOK Then
            mSymbolCount = mSymbolCount + 1
            attrs = CTrim(attrs)
            symbolID = CTrim(symbolID)

            'insert ENDOFPARA attribute if followed by para. mark (or end of table cell)
            Set aRange = .Range
            aRange.Collapse wdCollapseEnd
            If (aRange.EndOf(wdCharacter, 1) = 1) Then
                If Len(aRange.Text) > 0 Then
                    If (Asc(aRange.Text) = 13) Then
                        endOfPara = " ENDOFPARA=1"
                    End If
                End If
            End If
            Set aRange = Nothing

            tag = "[!DSTAG SYMBOL ID=ch" & Strings.format$(mSymbolCount, "0000") & " " & _
                attrs & " CHARID=ch" & symbolID & " INSUBSUPER=" & inSubSuper & _
                endOfPara & ktagEnd

            InsertTag tag

            ProcessSymbol = True
            MTIncrementStatisticBy "MPMMLSym", 1

        ElseIf stat = mtTRANSLATOR_ERROR Then
            'replace symbol with error message
            'and display a summary dialog at the end of processing
            .Text = MTLib.GetUserString2("1032", "1036", "***TRANSLATION ERROR***")
            mHasTranslatorErrors = True
        Else
            MsgBox MTLib.GetUserString("!1006An unexpected processing error occurred, please try again. If this continues to occur, try reinstalling MathType.") & vbCrLf & _
                MTLib.GetUserString("!1021Process Symbol error: ") & stat, vbCritical + vbOKOnly, kMPName
        End If
    End With
End Function

'Processes all equations in the document.
Private Function ProcessEquations() As Boolean
    Dim aStory As Range
    Dim aRange As Range
    Dim eqnCount As Long

    eqnCount = 1
    MTLib.SetActiveProgressString kPROGRESS_EQUATION

    For Each aStory In mDocument.StoryRanges
        'We skip floating objects & header/footers (cause problems in W97)...
        If IsValidStory(aStory) Then
            'Some stories (textboxes) actually contain a range of ranges...
            'so we need an extra loop
            Do While (IsObjectValid(aStory))
                Set aRange = aStory.Duplicate

                If Not ProcessEquationsInRange(aRange, eqnCount) Then
                    'error occurred
                    ProcessEquations = False
                    Exit Function
                End If

                'if this story has a valid NextRange, process it
                Set aStory = aStory.NextStoryRange
            Loop
        End If
    Next 'story
    ProcessEquations = True
End Function

Private Function ProcessEquationsInRange( _
   aRange As Range, _
   ByRef eqnCount As Long) As Boolean

   Dim i As Long
   Dim aShape As InlineShape
   Dim isEquation As Boolean
   Dim isDisplay As Boolean
   Dim tag As String
   Dim stat As Long
   Dim shapes() As InlineShape
   Dim numShapes As Long

   ProcessEquationsInRange = True

   If aRange.InlineShapes.count <= 0 Then
      Exit Function
   End If

   'ignore floating text boxes for now...
   If aRange.storyType = wdTextFrameStory Then
       'set 'floating eqns' flag & exit
       mHasFloatingEquations = True
       ProcessEquationsInRange = False
       Exit Function
   End If

   On Error GoTo err

   'copy all inline shapes into another array
   'Office XP: InlineShapes array has holes if are canvas objects present
   'also, we replace all equations with text which modifies the InlineShapes collection
   ReDim shapes(aRange.InlineShapes.count)
   numShapes = 0
   For Each aShape In aRange.InlineShapes
      numShapes = numShapes + 1
      Set shapes(numShapes) = aShape
   Next

   'loop through inline graphics
   For i = 1 To numShapes
      Set aShape = shapes(i)
      'if it is an OLE equation or a picture...
      isEquation = False
      If aShape.Type = wdInlineShapeEmbeddedOLEObject Then
         'accessing ProgID may cause an error 5825 on corrupt objects, so use ClassType instead
         isEquation = (MTLib.IsEquationProgID(aShape.OLEFormat.ClassType) <> 0)
      End If

      If Not isEquation Then
         isEquation = (aShape.Type = wdInlineShapePicture)
      End If

      If isEquation Then
         aShape.Select
         isDisplay = IsDisplayEquation(aShape)
         tag = Strings.Space$(256)
         stat = CreateGIF(aShape, eqnCount, isDisplay, tag)
         If (stat = mpOK) Or (stat = mpEQN_NO_BASELINE) Or (stat = mtTRANSLATOR_ERROR) Then
            FixWhiteSpace aShape
            'it was one of our equations, so replace with tag(s)
            If isDisplay Then
                InsertDisplayTags aShape
                're-assign as table insertion deletes object
                'Set aShape = aRange.InlineShapes(1) - not needed
            End If

            aShape.Select
            InsertTag tag
            eqnCount = eqnCount + 1   ' increment eqn# counter
            MTLib.ShowProgressString _
                MTLib.GetUserString("!1023Processing Equations (") & _
                eqnCount & MTLib.GetUserString("!1015)")
            If stat = mtTRANSLATOR_ERROR Then
                'insert error message after equation
                'and display a summary dialog at the end of processing
                If gOptions.insertTranslationError Then
                  Selection.InsertAfter MTLib.GetUserString2("1032", "1036", "***TRANSLATION ERROR***")
                End If
                mHasTranslatorErrors = True
            End If
         ElseIf stat <> mpNOT_AN_EQUATION Then
            'error occurred
            ProcessEquationsInRange = False
            Exit Function
         End If
         DoEvents    'let Windows update if necessary
      End If
   Next

   Exit Function

err:
   MsgBox MTLib.GetUserString("!1006An unexpected processing error occurred, please try again. If this continues to occur, try reinstalling MathType.") & vbCrLf & _
        "ProcessEquationsInRange: error " & err.Number & ", " & err.Description, vbCritical + vbOKOnly, kMPName
   ProcessEquationsInRange = False
End Function

'Inserts space char before/after eqn based on preceding and trailing chars
'If preceded by non-whitespace, insert a space
'If followed by non-whitespace, non-punctuation, insert a space
Private Sub FixWhiteSpace(aShape As InlineShape)
    Dim whiteSpace As Boolean
    Dim char As Integer
    Dim aRange As Range

    Set aRange = aShape.Range
    With aRange
        .Collapse wdCollapseStart
        If .StartOf(wdCharacter, wdExtend) = -1 Then
            If Len(.Text) > 0 Then
                If IsWhiteSpace(Asc(.Text)) = False Then
                    .InsertAfter " "
                End If
            End If
        End If
        .Collapse wdCollapseEnd
        .move wdCharacter, 1
        If .EndOf(wdCharacter, wdExtend) = 1 And Len(.Text) > 0 Then
            If (IsWhiteSpace(Asc(.Text)) = False) And (IsPunctuation(Asc(.Text)) = False) Then
                .InsertBefore " "
            End If
        End If
    End With
End Sub

'Returns True if the char is considered white space (space or non-breaking space)
Private Function IsWhiteSpace(char As Integer) As Boolean
    IsWhiteSpace = (char <= 32) Or (char = 160)
End Function

'Returns True if the char is considered punctuation
Private Function IsPunctuation(char As Integer) As Boolean
    ' punctuation = .,:;!?
    IsPunctuation = (char = Asc(".")) Or (char = Asc(",")) Or (char = Asc(";")) _
        Or (char = Asc(":")) Or (char = Asc("!")) Or (char = Asc("?"))
End Function

'Returns True if equation successfully converted to a GIF
Private Function CreateGIF( _
    eqnShape As InlineShape, _
    EqnNum As Long, _
    isDisplay As Boolean, _
    ByRef tag As String) _
    As Long

    Dim rc As Long
    Dim eqnID As String
    Dim attribs As String
    Dim info As GIFInfo2
    Dim singleLineSpacing As Boolean
    Dim endOfPara As String
    Dim aRange As Range

    rc = mpNOT_AN_EQUATION

    'select the shape
    eqnShape.Select

    singleLineSpacing = (gAppVersion = kWord97) Or isDisplay Or _
        (eqnShape.Range.ParagraphFormat.LineSpacingRule = wdLineSpaceSingle)

    With info
        .version = 1
        .smooth = gOptions.antiAlias
        .bkgndColor = GetEqnBackgroundColor(eqnShape)
        If eqnShape.Fill.visible Then
            .fillType = eqnShape.Fill.Type
        End If
        If isDisplay Then
            .display = mpdtDisplay
        Else
            .display = mpdtInline
        End If

        'get size from preceding character
        Set aRange = eqnShape.Range.Duplicate
        aRange.Collapse wdCollapseStart
        .size = aRange.font.size
        Set aRange = Nothing

        'if borders are on...
        If eqnShape.Borders.Enable Then
            SetBorderInfo eqnShape.Borders(wdBorderTop), .topBorder
            SetBorderInfo eqnShape.Borders(wdBorderLeft), .leftBorder
            SetBorderInfo eqnShape.Borders(wdBorderBottom), .bottomBorder
            SetBorderInfo eqnShape.Borders(wdBorderRight), .rightBorder
            'turn them off so metafile has no trace of the border...
            eqnShape.Borders.Enable = False
        End If
    End With

    'insert ENDOFPARA attribute in tag if followed by a paragraph mark
    Set aRange = eqnShape.Range.Duplicate
    With aRange
        .Collapse wdCollapseEnd
        If (.EndOf(wdCharacter, wdExtend) = 1) Then
            If Len(.Text) > 0 Then 'can't test Asc of null string
                If Asc(.Text) = 13 Then
                    endOfPara = " ENDOFPARA=1"
                End If
            End If
        End If
    End With
    Set aRange = Nothing

    'Place shape on clipboard
    'deleted text not shown on screen will generate an error 4198
    'deleted revisions cannot be copied to the clipboard...skip them
    If CopyToClipboard(eqnShape.Range) Then
        eqnID = Strings.format$(EqnNum, "0000")
        attribs = Strings.Space$(256)
        rc = MPProcessEquation2("eq" & eqnID & "<res>.gif", info, attribs)
        If rc = mpOK Or rc = mpEQN_NO_BASELINE Or rc = mtTRANSLATOR_ERROR Then
            tag = "[!DSTAG EQN ID=eq" & eqnID & endOfPara
            If singleLineSpacing Then
                tag = tag & " RESERVE_LINE_SPACING=1"
            End If
            attribs = CTrim(attribs)
            If Len(attribs) > 0 Then
                tag = tag & " " & attribs
            End If
            tag = tag & ktagEnd
        ElseIf rc = mpNOT_AN_EQUATION Then
            'do nothing here; pass error code back to caller
        Else
            MsgBox MTLib.GetUserString("!1006An unexpected processing error occurred, please try again. If this continues to occur, try reinstalling MathType.") & vbCrLf & _
                "MPProcessEquation2 returned error code " & rc, vbCritical + vbOKOnly, kMPName
        End If
    End If

    CreateGIF = rc
End Function

'attempt to copy the given range to the clipboard
Private Function CopyToClipboard(r As Range) As Boolean
    CopyToClipboard = False
    On Error GoTo err
    r.Copy
    CopyToClipboard = True
err:
End Function

' sets border information
Private Sub SetBorderInfo(aBorder As Border, ByRef info As BorderInfo)
    If aBorder.visible Then
        With info
            .width = aBorder.LineWidth
            .style = aBorder.LineStyle
            If gAppVersion = kWord97 Then
                .color = GetBorderColor97(aBorder)
            Else
                .color = GetBorderColor2000(aBorder)
            End If
        End With
    End If
End Sub

'Yet another W2000 function due to W97's simpler object model
Private Function GetBorderColor2000(aBorder As Border) As Long
    GetBorderColor2000 = aBorder.color
End Function

'Yet another W2000 function due to W97's simpler object model
Private Function GetBorderColor97(aBorder As Border) As Long
    GetBorderColor97 = ColorIndexToRGB(aBorder.ColorIndex)
End Function

'Determines if the equation shape is a display equation or not.
'We do not base the decision on the style, as an equation inserted as
'a display equation could be copied/dragged to an inline location.
'We consider it NOT TO BE a display equation if:
'   - there is more than 1 shape (graphic) on the line
'   - equation is inside of a table
'   - has style set to <kInlineEquationStyle>
'   - the paragraph is centered
'We consider it TO BE a display equation if the equation:
'   - has style set to <kDisplayEquationStyle>
'   - is preceded by <TAB>, AND followed by <CR> or <TAB>, AND
'       the 1st tab stop in the line is centered, AND
'       2nd tab stop (if any) is right-aligned at right margin

'Returns True if display
Private Function IsDisplayEquation(aShape As InlineShape) As Boolean

    Dim crPos As Long
    Dim tabPos As Long
    Dim paraRange As Range
    Dim aTab As TabStop
    Dim numCustomTabStops As Long

    IsDisplayEquation = False   'assume not a display equation

    aShape.Select
    With Selection
        If .Information(wdWithInTable) Then
            Exit Function   'equation is in a table, always treated as inline
        End If

        'check for user-inserted style over-rides
        If .style.NameLocal = kInlineEquationStyle Then
            Exit Function   'force inline
        ElseIf .style.NameLocal = kDisplayEquationStyle Then
            IsDisplayEquation = True
            Exit Function   'force display
        End If

        'check for multiple graphics (equations) on a line...
        Set paraRange = .Paragraphs(1).Range
        If paraRange.InlineShapes.count > 1 Then
            'not display if other graphics (equations) in same paragraph
            'can't easily test if pictures are eqns, so treat all graphics same way
            'seems unlikely that graphics will be on same line as display eqn, and
            'user can override them anyway
            Exit Function
        ElseIf InStr(1, paraRange.Text, "[!DSTAG EQN", vbBinaryCompare) > 0 Then
            'not display if we previously handled equations in same paragraph
            Exit Function
        End If

        'assume display if:
        ' - we're in a 1-line paragraph, AND
        ' - first tab stop in paragraph is a center tab stop, AND
        ' - a <TAB> character is before the equation
        ' - an optional 2nd tab stop is right-aligned
        If IsParagraphSingleLine() Then

            'if first tabstop is centered...
            If .ParagraphFormat.TabStops.count >= 1 And _
               .ParagraphFormat.TabStops(1).Alignment = wdAlignTabCenter Then

                'make sure optional 2nd tab is right-aligned at right margin
                If .ParagraphFormat.TabStops.count = 2 Then
                    If .ParagraphFormat.TabStops(2).Alignment <> wdAlignTabRight Or _
                       .ParagraphFormat.TabStops(2).position <> (.PageSetup.PageWidth - .PageSetup.RightMargin - .PageSetup.LeftMargin) Then
                        Exit Function '2nd of only 2 tabstops is not right-aligned
                    End If
                Else
                    'any additional tabstops must be default (non-custom) tabs
                    numCustomTabStops = 0
                    For Each aTab In .ParagraphFormat.TabStops
                        If aTab.CustomTab Then
                            numCustomTabStops = numCustomTabStops + 1
                        End If
                    Next
                    If numCustomTabStops > 1 Then
                        Exit Function '2nd custom tabstop must not be at right margin
                    End If
                End If

                'make sure there is a tab char before the equation
                aShape.Select
                crPos = .MoveStartUntil(vbCr, wdBackward)
                aShape.Select
                tabPos = .MoveStartUntil(vbTab, wdBackward)
                If (tabPos < 0) And ((crPos < tabPos) Or (crPos = 0)) Then
                    'equation paragraph met all of our conditions
                    IsDisplayEquation = True
                End If

            End If '1st tabstop is centered

        End If 'isParagraphSingleLine

    End With 'Selection
End Function

'Returns True if the selection is on a single-line paragraph
Private Function IsParagraphSingleLine() As Boolean
    Dim curLineBM As Bookmark
    Dim curParaBM As Bookmark

    IsParagraphSingleLine = False

    'If error occurs, bale & return False
    On Error GoTo abort

    'check if paragraph occupies a single line in the document
    'seems to be the best way to do this...
    With mDocument.Bookmarks
        Set curLineBM = .item(Strings.ChrW(&H5C) + "Line")
        Set curParaBM = .item(Strings.ChrW(&H5C) + "Para")
    End With
    If (curLineBM.start = curParaBM.start) And (curLineBM.end = curParaBM.end) Then
        IsParagraphSingleLine = True
    End If

abort:
End Function

'Inserts the DisplayEquation tags in the appropriate places
Private Function InsertDisplayTags(aShape As InlineShape) As Boolean
    Dim aRange As Range
    Dim crPos As Long
    Dim tabPos As Long

    InsertDisplayTags = False
    With Selection
        crPos = .MoveStartUntil(vbCr, wdBackward)
        ' if no cr found (1st line of doc), set start of selection to start of paragraph
        If crPos = 0 Then
            Selection.start = Selection.Paragraphs.First.Range.start
        End If
        .Collapse direction:=wdCollapseStart
        InsertTag ktagDisplayTable

        'find tab preceding the equation, insert directly before eqn if none found
        'must delete all sequential tabs to prevent Word generating runs of &nbsp;
        aShape.Select
        crPos = .MoveStartUntil(vbCr, wdBackward)
        ' if no cr found (1st line of doc), set start of selection to start of paragraph
        If crPos = 0 Then
            Selection.start = Selection.Paragraphs.First.Range.start
        End If
        aShape.Select
        If crPos = 0 Then
            tabPos = .MoveStartUntil(vbTab, wdBackward)
        Else
            tabPos = .MoveStartUntil(vbTab, crPos)
        End If
        If (tabPos = 0) Or ((tabPos < crPos) And (crPos <> 0)) Then
            aShape.Select   'no tabs after preceding CR
            .InsertBefore GetTableSeparator
        Else
            'search for and delete any preceding sequential tabs
            .Collapse direction:=wdCollapseStart
            .InsertBefore GetTableSeparator
            .Collapse direction:=wdCollapseStart
            DeleteSelectionWhile vbTab, False
        End If

        'find tab following equation, insert separator before the tab
        aShape.Select
        crPos = .MoveEndUntil(vbCr, wdForward)
        aShape.Select
        tabPos = .MoveEndUntil(vbTab, wdForward)
        'if no tab found before cr, insert separator immediately before the cr
        If (tabPos = 0) Or (tabPos > crPos) Then
            aShape.Select
            .MoveEndUntil vbCr, wdForward
        Else
            'search for and delete any trailing sequential tabs
            .Collapse direction:=wdCollapseEnd
            DeleteSelectionWhile vbTab, True
        End If
        Selection.InsertAfter GetTableSeparator

        'insert table around paragraph
        aShape.Select
        crPos = .MoveStartUntil(vbCr, wdBackward)
        ' if no cr found (1st line of doc), set start of selection to start of paragraph
        If crPos = 0 Then
            Selection.start = Selection.Paragraphs.First.Range.start
        End If
        .MoveEndUntil vbCr, wdForward
        InsertDisplayEquationTable aShape
        InsertDisplayTags = True
    End With
End Function

'Inserts a table wrapping the existing selection. Uses GetTableSeparator as the cell separator.
Private Sub InsertDisplayEquationTable(aShape As InlineShape)
    Dim aTable As Table
    Dim leftIndent As Single

    leftIndent = Selection.ParagraphFormat.leftIndent

    Set aTable = Selection.ConvertToTable(separator:=GetTableSeparator, NumColumns:=3, _
        numRows:=1, format:=wdTableFormatNone, ApplyBorders:=False, ApplyShading:=True, _
        ApplyFont:=False, ApplyColor:=True, ApplyHeadingRows:=False, ApplyLastRow:=False, _
        ApplyFirstColumn:=False, ApplyLastColumn:=False, AutoFit:=True)

    'shape object is deleted as part of converting to a table; re-assign
    Set aShape = Selection.Cells(2).Range.InlineShapes(1)

    aTable.Borders.Enable = False 'hide borders

    If gAppVersion > kWord97 Then
        SetTableParams2000 aTable
    End If
    aTable.Rows.leftIndent = leftIndent
End Sub

'Word 2000 only function, sets width of table to document width etc.
Private Function SetTableParams2000(aTable As Table)
    aTable.AutoFitBehavior wdAutoFitWindow
End Function

'Deletes current selection in given direction while each new character matches whileChar
'Pass True in direction for forwards, False for backwards
'Returns True if selection extended OK, false if extending at some point failed (e.g. end of doc)
Private Function DeleteSelectionWhile(whileChar As String, direction As Boolean) As Boolean
    Dim stat As Long

    DeleteSelectionWhile = False

    With Selection
        If direction Then
            Do While (.MoveEnd(wdCharacter, 1) <> 0)
                If Strings.right$(.Text, 1) <> whileChar Then
                    .Collapse wdCollapseStart
                    DeleteSelectionWhile = True
                    Exit Do
                Else
                    .delete
                End If
            Loop
        Else
            Do While (.MoveStart(wdCharacter, -1) <> 0)
                If Strings.left$(.Text, 1) <> whileChar Then
                    .Collapse wdCollapseEnd
                    DeleteSelectionWhile = True
                    Exit Do
                Else
                    .delete
                End If
            Loop
        End If
    End With
End Function

'Return background color of equation by looking for the following:
'   use InlineShape's Fill foreColor if the Fill is visible & solid
'   use shape's Font object's background pattern if not textured and not 'auto'
'   use shape's range's Cells object's background pattern if not textured and not 'auto'
'   use shape's Table(1) object's background pattern if not textured and not 'auto'
'   use shape's ParagraphFormat object's background pattern if not textured and not 'auto'
'   use document background's Fill forecolor if the Fill is visible & solid
'Returns FFFFFF (white) if no explicit color found
Private Function GetEqnBackgroundColor(eqnShape As InlineShape) As Long
    Dim eqnColor As Long
    Dim eqnColorSet As Boolean

    eqnColor = &HFFFFFF     'default to White
    eqnColorSet = False

    'Note: have seen this fail once with a 4198 error - Command Failed
    'Not sure if the equation was 'old' or somehow damaged - it wasn't showing up
    With eqnShape.Fill
        If .visible And (.Type = msoFillSolid) Then
            eqnColor = .ForeColor.RGB
            eqnColorSet = True
        End If
    End With

    With eqnShape.Range
        If Not eqnColorSet Then
            eqnColorSet = GetShadingColor(.font.Shading, eqnColor)
        End If

        If Not eqnColorSet And (.Tables.count > 0) Then
            If Not eqnColorSet And (.Cells.count > 0) Then
                eqnColorSet = GetShadingColor(.Cells.Shading, eqnColor)
            End If
            If Not eqnColorSet Then
                eqnColorSet = GetShadingColor(.Tables(1).Shading, eqnColor)
            End If
        End If

        If Not eqnColorSet Then
            eqnColorSet = GetShadingColor(.ParagraphFormat.Shading, eqnColor)
        End If
    End With

    If Not eqnColorSet Then
        With mDocument.Background.Fill
            If .visible And (.Type = msoFillSolid) Then
                eqnColor = .ForeColor.RGB
                eqnColorSet = True
            End If
        End With
    End If

    'MsgBox "RGB = " & (eqnColor And 255) & "," & ((eqnColor And 65280) \ 256) & "," & (eqnColor \ 65536)
    GetEqnBackgroundColor = eqnColor
End Function

'Gets background color from Shading object
'Returns True if a color was found
Private Function GetShadingColor(shade As Shading, ByRef eqnColor As Long) As Boolean
    If gAppVersion = kWord97 Then
        GetShadingColor = GetShadingColor97(shade, eqnColor)
    Else
        GetShadingColor = GetShadingColor2000(shade, eqnColor)
    End If
End Function

Private Function GetShadingColor2000(shade As Shading, ByRef eqnColor As Long) As Boolean
    Dim eqnColorSet As Boolean

    eqnColorSet = False
    With shade
        If .BackgroundPatternColor <> wdColorAutomatic Then
            If .Texture = wdTextureNone Then
                eqnColor = .BackgroundPatternColor
                eqnColorSet = True
            Else
                Dim textureVal As Single

                'Our 'clumsy' approx. to Word's texture shading algorithm
                'Actually this assumes shading to black
                'In Word the shading color can be any color too!
                If (.Texture > 10) And (.Texture <= 1000) Then
                    Dim r As Integer, g As Integer, b As Integer
                    textureVal = 1 - (.Texture / 1000) ' 100% = 0
                    r = (.BackgroundPatternColor And 255) * textureVal
                    g = ((.BackgroundPatternColor And 65280) \ 256) * textureVal
                    b = (.BackgroundPatternColor \ 65536) * textureVal
                    eqnColor = RGB(r, g, b)
                    eqnColorSet = True
                End If
            End If
        End If
    End With
    GetShadingColor2000 = eqnColorSet
End Function

'Word97 version - Shading.BackgroundPatternColor not supported
Private Function GetShadingColor97(shade As Shading, ByRef eqnColor As Long) As Boolean
    Dim eqnColorSet As Boolean

    eqnColorSet = False
    With shade
        If .BackgroundPatternColorIndex <> wdAuto Then
            eqnColor = ColorIndexToRGB(.BackgroundPatternColorIndex)
            If .Texture = wdTextureNone Then
                eqnColorSet = True
            ElseIf .Texture > 10 Then
                Dim textureVal As Single
                Dim r As Integer, g As Integer, b As Integer

                'Our 'clumsy' approx. to Word's shading algorithm
                '(Not really sure if Word97 needs all this)
                textureVal = 1 - (.Texture / 1000) ' 100% = 0
                r = (eqnColor And 255) * textureVal
                g = ((eqnColor And 65280) \ 256) * textureVal
                b = (eqnColor \ 65536) * textureVal
                eqnColor = RGB(r, g, b)
                eqnColorSet = True
            End If
        End If
    End With
    GetShadingColor97 = False
End Function
'Converts wdColorIndex to RGB value
Private Function ColorIndexToRGB(index As Long) As Long
    Select Case index
    Case wdBlack
        ColorIndexToRGB = RGB(0, 0, 0)
    Case wdBlue
        ColorIndexToRGB = RGB(0, 0, 255)
    Case wdBrightGreen
        ColorIndexToRGB = RGB(0, 255, 0)
    Case wdDarkBlue
        ColorIndexToRGB = RGB(0, 0, 128)
    Case wdDarkRed
        ColorIndexToRGB = RGB(128, 0, 0)
    Case wdDarkYellow
        ColorIndexToRGB = RGB(128, 128, 0)
    Case wdGray25
        ColorIndexToRGB = RGB(192, 192, 192)
    Case wdGray50
        ColorIndexToRGB = RGB(128, 128, 128)
    Case wdGreen
        ColorIndexToRGB = RGB(0, 128, 0)
    Case wdPink
        ColorIndexToRGB = RGB(255, 0, 255)
    Case wdRed
        ColorIndexToRGB = RGB(255, 0, 0)
    Case wdTeal
        ColorIndexToRGB = RGB(0, 128, 128)
    Case wdTurquoise
        ColorIndexToRGB = RGB(0, 255, 255)
    Case wdViolet
        ColorIndexToRGB = RGB(128, 0, 128)
    Case wdYellow
        ColorIndexToRGB = RGB(255, 255, 0)
    Case wdWhite
        ColorIndexToRGB = RGB(255, 255, 255)
    Case Else   'default to Black
        ColorIndexToRGB = RGB(0, 0, 0)
    End Select
End Function

'Returns True if HTML convert found (Word97 only, built-in for Word2000)
Private Function HTMLConverterFound() As Boolean
    Dim aConverter As FileConverter
    Dim formatFound As Boolean

    formatFound = False

    'find ID of HTML converter
    For Each aConverter In FileConverters
        If aConverter.ClassName = "HTML" Then
            gHTMLFormatID = aConverter.SaveFormat
            formatFound = True
            Exit For
        End If
    Next aConverter

    HTMLConverterFound = formatFound
End Function

'Word2000 (& newer) version
Private Function GetFontColor2000(aFont As font) As Long
    If aFont.color <> wdColorAutomatic Then
        GetFontColor2000 = aFont.color
    Else
        GetFontColor2000 = 0    'is auto always black?
    End If
End Function

'Word97 version
Private Function GetFontColor97(aFont As font) As Long
    If aFont.ColorIndex <> wdAuto Then
        GetFontColor97 = ColorIndexToRGB(aFont.ColorIndex)
    Else
        GetFontColor97 = 0    'is auto always black?
    End If
End Function

Private Sub InitFindFont(ByRef aFind As find, fontName As String)
    InitFind aFind
    aFind.font.name = fontName
End Sub

Private Sub InitFind(ByRef aFind As find)
    With aFind
        .ClearFormatting
        .font.name = ""
        .Text = ""
        .Replacement.Text = ""
        .forward = True
        .Wrap = wdFindStop  'searching whole doc, so stop at end
        .format = True
        .MatchCase = False
        .MatchWholeWord = False
        .MatchAllWordForms = False
        .MatchSoundsLike = False
        .MatchWildcards = False
        If gAppVersion > kWord97 Then
            InitFind2000 aFind
        End If
    End With
 End Sub

Private Function InitFind2000(ByRef aFind As find)
    With aFind
        .MatchFuzzy = False
        .MatchByte = False
    End With
End Function

'Returns id of StoryRange containing the selection, or -1 if not found
Private Function FindStory(Doc As Document, storyType As Long, aRange As Range) As Integer
    Dim searchRange As Range

    FindStory = 0
    Set searchRange = Doc.StoryRanges(storyType)

    On Error GoTo Erred
    While Not (searchRange.InStory(aRange))
        Set searchRange = searchRange.NextStoryRange
        FindStory = FindStory + 1
    Wend
    Exit Function
Erred:
    FindStory = -1
End Function

'Returns Word2000's folder suffix for support files (not in Word97)
Private Function GetFolderSuffix() As String
    GetFolderSuffix = Application.DefaultWebOptions.FolderSuffix
End Function

'Returns string used by Word to represent "(normal text)" in the InsertSymbol dialog.
'It's localized but not a constant or obtainable via an API.
'To get it we temporarily insert some known text at the start of the doc and test its font.
Private Function GetNormalText(Doc As Document)
    Dim dlg As Dialog
    Dim oldSel As Range

    With Selection
        Set oldSel = .Range
        .start = 0
        .end = 0
        .InsertAfter "b " & vbCrLf
        .style = Doc.Styles(wdStyleNormal)
    End With
    Set dlg = Dialogs(wdDialogInsertSymbol)
    GetNormalText = dlg.font
    Doc.Undo 2
    oldSel.Select
End Function

'Utility function to trim extra spaces from a string & then remove terminating char (null)
'Useful for C-style strings returned from DLLs
Public Function CTrim(cstring As String) As String
    If Len(cstring) > 0 Then
        CTrim = Strings.Trim$(cstring)
        If Len(CTrim) > 0 Then
            CTrim = Strings.left$(CTrim, Len(CTrim) - 1)
        End If
    End If
End Function

'Returns True if Selection contains grouped items
Function IsGrouped() As Boolean
    IsGrouped = False

    On Error GoTo err
    IsGrouped = (Selection.ShapeRange.GroupItems.count > 1)
err:
End Function

'Return W2000 HTML format ID
Function GetW2000HTMLFormatID() As Long
    GetW2000HTMLFormatID = wdFormatHTML
End Function

'Writes a string custom document property
'Handles case where prop may or may not exist
Public Sub WriteStringDocProperty(Doc As Document, ByRef propName As String, _
    ByRef propValue As String)
    On Error Resume Next
    Doc.CustomDocumentProperties(propName).delete

    Doc.CustomDocumentProperties.Add propName, False, _
        msoPropertyTypeString, propValue
    On Error GoTo 0
End Sub

'Writes a string custom document property
'Handles case where prop may or may not exist
Public Sub WriteBoolDocProperty(Doc As Document, ByRef propName As String, _
    propValue As Boolean)
    On Error Resume Next
    Doc.CustomDocumentProperties(propName).delete

    Doc.CustomDocumentProperties.Add propName, _
           False, msoPropertyTypeBoolean, propValue
    On Error GoTo 0
End Sub

'separator used to identify cells in display equation table
Private Function GetTableSeparator() As String
    GetTableSeparator = Strings.Chr(163)
End Function
