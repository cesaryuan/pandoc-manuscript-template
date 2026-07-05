Attribute VB_Name = "MTConvertEquationsDlg"
Attribute VB_Base = "0{BBDF7063-C0D6-49E0-889C-9DE36339D28D}{F5972B8C-4314-4DB4-9723-5B45A26ADB4C}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False




'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/MTConvertEquationsDlg.frm 1     5/06/14 9:55a Jimm $
'=====================================================================

Option Explicit

Private transDescriptions$(), transFiles$(), transNames$(), transInfo$()

Private Sub btn_Help_Click()
   OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin#convert_equations_dialog"
'    MTLib.MTHelpTopic hlpMSWDConvert_Equations_Dialog
End Sub

Private Function TranslatorHasOptions()
    If (cmb_Translator.value = "AMSLaTeX" Or _
        cmb_Translator.value = "AMSTeX" Or _
        cmb_Translator.value = "LaTeX 2.09 and later" Or _
        cmb_Translator.value = "MathML 2.0 (m namespace)" Or _
        cmb_Translator.value = "MathML 2.0 (namespace attr)" Or _
        cmb_Translator.value = "MathML 2.0 (no namespace)" Or _
        cmb_Translator.value = "Plain TeX" Or _
        cmb_Translator.value = "Texvc (LaTeX delimiters)") Then
        TranslatorHasOptions = True
    Else
        TranslatorHasOptions = False
    End If
End Function

Private Sub cmb_Translator_Change()
    On Error Resume Next
    'change the Description and File labels to match the selected translator
    lbl_TransDesc.caption = transDescriptions$(cmb_Translator.ListIndex)
    lbl_TransFile.caption = transFiles$(cmb_Translator.ListIndex)
    'Enable all the Translator options and select Text Translators
    '(in case they weren't set that way before)
    opt_toTex.value = True
    lbl_TransDesc.enabled = True
    lbl_TransFile.enabled = True
    If TranslatorHasOptions() Then
    chk_TransName.enabled = True
    chk_TransMTEF.enabled = True
    Else
        chk_TransName.enabled = False
        chk_TransMTEF.enabled = False
    End If
End Sub

'Disable all the Translator options
Private Sub opt_toMathType_Click()
    lbl_TransDesc.enabled = False
    lbl_TransFile.enabled = False
    chk_TransName.enabled = False
    chk_TransMTEF.enabled = False
    cmb_Translator.enabled = False
End Sub

'Enable all the Translator options
Private Sub opt_toTex_Click()
    lbl_TransDesc.enabled = True
    lbl_TransFile.enabled = True
    If TranslatorHasOptions() Then
    chk_TransName.enabled = True
    chk_TransMTEF.enabled = True
    Else
        chk_TransName.enabled = False
        chk_TransMTEF.enabled = False
    End If
    cmb_Translator.enabled = True
End Sub

Private Sub UserForm_Initialize()
    Dim myControl As MSForms.control
    Dim settings
    Dim misc$
    Dim fontName$
    Dim fontSize As Long

    'assign the dialog's font & size according to the current language
    '#If Mac Then
    'Me.BackColor = &H8000000F
    'fontName$ = MTLib.GetUserString("!0010Lucida Grande")
    'fontSize = Val(MTLib.GetUserString("!001111"))
    'Me.height = 5280
    'Me.width = 12930
    '#Else ' if Win
    'fontName$ = MTLib.GetUserString("!0010Tahoma")
    'fontSize = Val(MTLib.GetUserString("!00118.25"))
    'Me.height = 202.75  '3435
    'Me.width = 500.5    '9720
    '#End If
    'Me.font.name = fontName$
    'Me.font.size = fontSize'

    'assign the captions according to the current language
    Me.caption = MTLib.GetUserString(Me.caption)
    For Each myControl In Me.Controls
        'myControl.font.name = fontName$
        'myControl.font.size = fontSize
        If myControl.name <> "cmb_Translator" _
           And myControl.name <> "lbl_TransDesc" _
           And myControl.name <> "lbl_TransFile" Then
            myControl.caption = MTLib.GetUserString(myControl.caption)
        End If
        #If Mac Then
        If TypeOf myControl Is MSForms.Frame Then
            myControl.BackColor = &H8000000F
        End If
        #End If
    Next myControl

    'initialize 'Convert From' checkboxes, get prev. settings from registry
    settings = Val(GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_CONVFROM))
    If (settings <= 0) Or (settings > 15) Then settings = 3
    chk_fmMathType.value = ((settings And 1) <> 0)
    chk_fmFormula.value = ((settings And 2) <> 0)
    chk_fmTex.value = ((settings And 4) <> 0)
    chk_fmOMML.value = ((settings And 8) <> 0)

    'load translator list
    settings = Val(GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_CONVTRANS))
    LoadTranslators settings

    'initialize 'Convert To' checkboxes, get prev. settings from registry
    settings = Val(GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_CONVTO))
    'LoadTranslators disables Tex button if no translators found
    If (settings = 2) And (opt_toTex.enabled) Then
        opt_toTex.value = True
    Else
        opt_toMathType.value = True
    End If

    'initialize 'Convert' misc. checkboxes, get prev. settings from registry
    settings = -1
    misc$ = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_CONVMISC)
    If misc$ <> "" Then
        settings = Val(misc$)
    End If
    'default is include both translator comments, no prompting
    If (settings < 0) Or (settings > 7) Then settings = 3
    chk_TransMTEF.value = ((settings And 1) <> 0)
    chk_TransName.value = ((settings And 2) <> 0)
    chk_prompt.value = ((settings And 4) <> 0)

    'initialize the selection settings
    If Selection.Type = wdSelectionIP Then
        opt_rSelection.enabled = False
        opt_rWhole.value = True
    Else
        opt_rSelection.value = True
    End If

    'enable disable Word 2007 (OMML) equations
    Dim convertOMMLOkay As Boolean
    convertOMMLOkay = hasOMMLSupport()
    chk_fmOMML.enabled = convertOMMLOkay
    If Not convertOMMLOkay Then
        chk_fmOMML.value = False
    End If

    MTConvertEquations.gDlgCanceled = True
    Application.Activate 'in case MT had to be started up, grab the focus back
End Sub

Private Sub Cancel_Click()
    MTConvertEquations.gDlgCanceled = True
    unload Me
End Sub

Private Sub OK_Click()
    'make sure at least one "from" equation type is selected
    If Not (chk_fmMathType.value) And _
        Not (chk_fmFormula.value) And _
        Not (chk_fmTex.value) And _
        Not (chk_fmOMML.value) Then
        MsgBox MTLib.GetUserString("!0118You must pick at least one equation type to convert."), vbCritical, MTLib.GetUserString("!0100Convert Equations")
        Exit Sub
    End If

    'save settings in registry
    SaveSettings

    'Return the types to convert
    MTConvertEquations.gFindMathType = chk_fmMathType.value
    MTConvertEquations.gFindFields = chk_fmFormula.value
    MTConvertEquations.gFindText = chk_fmTex.value
    MTConvertEquations.gFindOMML = chk_fmOMML.value

    'Return the range
    If opt_rWhole.value Then
        MTConvertEquations.gUpdateRange = mt_RANGE_DOCUMENT
    Else
        MTConvertEquations.gUpdateRange = mt_RANGE_SELECTION
    End If

    'return the Prompt value
    MTConvertEquations.gPrompt = chk_prompt.value

    'If a translator is selected, return it, otherwise return an empty string
    'to indicate converting to MathType equations.
    If opt_toTex.value Then
        MTConvertEquations.gTransName$ = transFiles$(cmb_Translator.ListIndex)
        MTConvertEquations.gTransOptions = 0
        If chk_TransName.value Then
            MTConvertEquations.gTransOptions = MTConvertEquations.gTransOptions + 1
        End If
        If chk_TransMTEF.value Then
            MTConvertEquations.gTransOptions = MTConvertEquations.gTransOptions + 2
        End If
    Else
        MTConvertEquations.gTransName$ = ""
        MTConvertEquations.gTransOptions = 0
    End If

    MTConvertEquations.gDlgCanceled = False
    unload Me
End Sub

'saves dialog settings in registry for next time
Private Sub SaveSettings()
    Dim setting As Long

    'save 'From' setting
    setting = 0
    If chk_fmMathType.value Then setting = setting + 1
    If chk_fmFormula.value Then setting = setting + 2
    If chk_fmTex.value Then setting = setting + 4
    If chk_fmOMML.value Then setting = setting + 8
    SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_CONVFROM, Strings.LTrim(Conversion.str$(setting))

    'save 'To' setting and translator index
    setting = 0
    If opt_toTex.value Then
        setting = 2
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_CONVTRANS, Strings.LTrim(Conversion.str$(cmb_Translator.ListIndex))
    Else
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_CONVTRANS, "0"
    End If
    SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_CONVTO, Strings.LTrim(Conversion.str$(setting))

    'save 'Misc' setting
    setting = 0
    If chk_TransMTEF.value Then setting = setting + 1
    If chk_TransName.value Then setting = setting + 2
    If chk_prompt.value Then setting = setting + 4
    SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_CONVMISC, Strings.LTrim(Conversion.str$(setting))
End Sub
'loads translator list, and sets initial item based on index
Private Sub LoadTranslators(index)
    Dim NumTranslators
    Dim trNameSize As Integer, trDescSize As Integer, trFileSize As Integer
    Dim myIndex As Integer, nextIndex As Integer
    Dim curTransName$, curTransDesc$, curTransFile$
    Const kDelimitter As String = "|||"

    'initialize the translators
    NumTranslators = MTGetTranslatorsInfo(mttrnCOUNT)
    trNameSize = MTGetTranslatorsInfo(mttrnMAX_NAME)
    trDescSize = MTGetTranslatorsInfo(mttrnMAX_DESC)
    trFileSize = MTGetTranslatorsInfo(mttrnMAX_FILE)

    'if we actually have any translators, load the array
    If NumTranslators > 0 Then
        ReDim transDescriptions$(NumTranslators - 1)
        ReDim transFiles$(NumTranslators - 1)
        ReDim transNames$(NumTranslators - 1)
        ReDim transInfo(NumTranslators - 1)

        myIndex = 1
        'get each name, file and description
        While myIndex > 0
            curTransName$ = Strings.Space(trNameSize + 1)
            curTransDesc$ = Strings.Space(trDescSize + 1)
            curTransFile$ = Strings.Space(trFileSize + 1)
            nextIndex = MTEnumTranslators(myIndex, curTransName$, trNameSize, curTransDesc$, trDescSize, curTransFile$, trFileSize)

            curTransName = RemoveNull(curTransName)
            curTransDesc = RemoveNull(curTransDesc)
            curTransFile = RemoveNull(curTransFile)

            'If nextIndex < 0 we have an error condition. fail nicely
            If nextIndex <= 0 Then
                myIndex = nextIndex
            Else
                'combine the 3 values so they can be sorted
                transInfo(myIndex - 1) = curTransName + kDelimitter + curTransDesc + kDelimitter + curTransFile
                myIndex = nextIndex
            End If
        Wend

        'sort the array
        QuickSort transInfo, LBound(transInfo), UBound(transInfo)

        'add the translator names to the list box
        For myIndex = LBound(transInfo) To UBound(transInfo)
            Dim pos1, pos2 As Integer
            Dim str As String

            'name
            pos1 = InStr(1, transInfo(myIndex), kDelimitter) - 1
            str = Strings.left(transInfo(myIndex), pos1)
            transNames(myIndex) = str

            'description
            pos1 = pos1 + Len(kDelimitter) + 1
            pos2 = InStr(pos1, transInfo(myIndex), kDelimitter)
            str = Strings.Mid(transInfo(myIndex), pos1, pos2 - pos1)
            transDescriptions(myIndex) = str

            'file
            str = Strings.right(transInfo(myIndex), Len(transInfo(myIndex)) - pos2 - Len(kDelimitter) + 1)
            transFiles(myIndex) = str

            cmb_Translator.addItem transNames(myIndex), myIndex
        Next myIndex

        'assign these values to the dialog
        If (index < 0) Or (index > (NumTranslators - 1)) Then index = 0
        On Error Resume Next
        cmb_Translator.ListIndex = index
        lbl_TransDesc.caption = transDescriptions$(index)
        lbl_TransFile.caption = transFiles$(index)
    Else
        'if there are no translators, clear all the strings and disable this section
        ReDim transDescriptions$(0)
        ReDim transFiles$(0)
        cmb_Translator.addItem "", 0
        transDescriptions$(0) = ""
        transFiles$(0) = ""
        opt_toTex.enabled = False
        opt_toMathType.value = True
        cmb_Translator.enabled = False
        lbl_TransDesc.enabled = False
        lbl_TransFile.enabled = False
        chk_TransName.enabled = False
        chk_TransMTEF.enabled = False
    End If

End Sub
