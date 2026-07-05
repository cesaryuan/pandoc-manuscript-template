Attribute VB_Name = "MTCommandsDispatchCls"
Attribute VB_Base = "0{FCFB3D2A-A0FA-1068-A738-08002B3371B5}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = True
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False

' Dispatch to MTBrowse

Public Sub MTCommandsMain_MTBrowse_BrowseEquationsForward()
    MTBrowse.BrowseEquationsForward
End Sub

Public Sub MTCommandsMain_MTBrowse_BrowseEquationNumberRefForward()
    MTBrowse.BrowseEquationNumberRefForward
End Sub

Public Sub MTCommandsMain_MTBrowse_BrowseChapterSectionForward()
    MTBrowse.BrowseChapterSectionForward
End Sub

Public Sub MTCommandsMain_MTBrowse_BrowseEquationsBackward()
    MTBrowse.BrowseEquationsBackward
End Sub

Public Sub MTCommandsMain_MTBrowse_BrowseEquationNumberRefBackward()
    MTBrowse.BrowseEquationNumberRefBackward
End Sub

Public Sub MTCommandsMain_MTBrowse_BrowseChapterSectionBackward()
    MTBrowse.BrowseChapterSectionBackward
End Sub

' Dispatch to  MTInsertEquation

Public Sub MTCommandsMain_MTInsertEquation_InsertInlineEquation()
    MTLib.WriteLog "dispatching to MTInsertEquation.InsertInlineEquation"
    MTInsertEquation.InsertInlineEquation
End Sub

Public Sub MTCommandsMain_MTInsertEquation_InsertDisplayEquation()
    MTInsertEquation.InsertDisplayEquation
End Sub

Public Sub MTCommandsMain_MTInsertEquation_InsertLeftNumberedDisplayEquation()
    MTInsertEquation.InsertLeftNumberedDisplayEquation
End Sub

Public Sub MTCommandsMain_MTInsertEquation_InsertRightNumberedDisplayEquation()
    MTInsertEquation.InsertRightNumberedDisplayEquation
End Sub

' Dispatch to MTEqnNum

Public Sub MTCommandsMain_MTEqnNum_DlgMain()
    MTEqnNum.DlgMain
End Sub

' Dispatch to MTEqnNumFormat

Public Sub MTCommandsMain_MTEqnNumFormat_DlgMain()
    MTEqnNumFormat.DlgMain
End Sub

' Dispatch to MTMarkRef

Public Sub MTCommandsMain_MTMarkRef_DlgMain()
    MTMarkRef.DlgMain
End Sub

' Dispatch to MTPlaceRef

Public Sub MTCommandsMain_MTPlaceRef_DlgMain()
    MTPlaceRef.DlgMain
End Sub

' Dispatch to MTSecNum

Public Sub MTCommandsMain_MTSecNum_DlgMain()
    MTSecNum.DlgMain
End Sub

Public Sub MTCommandsMain_MTSecNum_InsertNextSectionBreak()
    MTSecNum.InsertNextSectionBreak
End Sub

Public Sub MTCommandsMain_MTSecNum_InsertNextChapterBreak()
    MTSecNum.InsertNextChapterBreak
End Sub

' Dispatch to MTFormatEquationSection

Public Sub MTCommandsMain_MTFormatEquationSection_EditEquationSection()
    MTFormatEquationSection.EditEquationSection
End Sub

Public Sub MTCommandsMain_MTFormatEquationSection_NewVersion()
    MTFormatEquationSection.NewVersion
End Sub

Public Sub MTCommandsMain_MTFormatEquationSection_DlgMain()
    MTFormatEquationSection.DlgMain
End Sub

' Dispatch to MTFormatEquations

Public Sub MTCommandsMain_MTFormatEquations_DlgMain()
    MTFormatEquations.DlgMain
End Sub

' Dispatch to MTConvertEquations

Public Sub MTCommandsMain_MTConvertEquations_DlgMain()
    MTConvertEquations.DlgMain
End Sub

' Dispatch to MTExportEquations

Public Sub MTCommandsMain_MTExportEquations_DlgMain()
    MTExportEquations.DlgMain
End Sub

' Dispatch to MTUpdateFields

Public Sub MTCommandsMain_MTUpdateFields_DlgMain()
    MTUpdateFields.DlgMain
End Sub

' Dispatch to MTHelp

Public Sub MTCommandsMain_MTHelp_MTHelpTopicViaRegistry()
    MTHelp.MTHelpTopicViaRegistry
End Sub

' Dispatch to MathPage

Public Sub MTCommandsMain_MathPage_ExportAll()
    MathPage.ExportAll
End Sub

Public Sub MTCommandsMain_MathPage_MP_ExportTo()
    MathPage.MP_ExportTo
End Sub

' Dispatch to MTSetEqnPrefs

Public Sub MTCommandsMain_MTSetEqnPrefs_DlgMain()
    MTSetEqnPrefs.DlgMain
End Sub

' Dispatch to MTLib

Public Sub MTCommandsMain_MTLib_MTEditPicture()
    MTLib.MTEditPicture
End Sub

Public Sub MTCommandsMain_MTLib_MTCleanup()
    MTLib.MTCleanup
End Sub

Public Sub MTCommandsMain_MTTeXToggle_DlgMain()
    MTTeXToggle.DlgMain
End Sub

Public Sub MTCommandsMain_MTLib_MTEditEquationOpen()
    MTLib.MTEditEquation False
End Sub

Public Sub MTCommandsMain_MTLib_MTEditEquationInPlace()
    MTLib.MTEditEquation True
End Sub

Public Sub MTCommandsMain_MTMathInputControl_DlgMain()
    MTMathInputControl.DlgMain
End Sub

Public Sub MTCommandsMain_MTOptions_DlgMain()
    MTOptions.DlgMain
End Sub

Public Sub MTCommandsMain_MTLib_MTEditPaste()
    MTLib.MTEditPaste
End Sub
