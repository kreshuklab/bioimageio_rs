use aspartial::AsPartial;
use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(
    Default, Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone, strum::VariantArray, strum::VariantNames, strum::Display
)]
pub enum LicenseId {
    #[serde(rename = "0BSD")]
    ZERO_BSD,
    #[serde(rename = "AAL")]
    AAL,
    #[serde(rename = "Abstyles")]
    Abstyles,
    #[serde(rename = "AdaCore-doc")]
    AdaCore_doc,
    #[serde(rename = "Adobe-2006")]
    Adobe_2006,
    #[serde(rename = "Adobe-Glyph")]
    Adobe_Glyph,
    #[serde(rename = "ADSL")]
    ADSL,
    #[serde(rename = "AFL-1.1")]
    AFL_1_1,
    #[serde(rename = "AFL-1.2")]
    AFL_1_2,
    #[serde(rename = "AFL-2.0")]
    AFL_2_0,
    #[serde(rename = "AFL-2.1")]
    AFL_2_1,
    #[serde(rename = "AFL-3.0")]
    AFL_3_0,
    #[serde(rename = "Afmparse")]
    Afmparse,
    #[serde(rename = "AGPL-1.0-only")]
    AGPL_1_0_only,
    #[serde(rename = "AGPL-1.0-or-later")]
    AGPL_1_0_or_later,
    #[serde(rename = "AGPL-3.0-only")]
    AGPL_3_0_only,
    #[serde(rename = "AGPL-3.0-or-later")]
    AGPL_3_0_or_later,
    #[serde(rename = "Aladdin")]
    Aladdin,
    #[serde(rename = "AMDPLPA")]
    AMDPLPA,
    #[serde(rename = "AML")]
    AML,
    #[serde(rename = "AMPAS")]
    AMPAS,
    #[serde(rename = "ANTLR-PD")]
    ANTLR_PD,
    #[serde(rename = "ANTLR-PD-fallback")]
    ANTLR_PD_fallback,
    #[serde(rename = "Apache-1.0")]
    Apache_1_0,
    #[serde(rename = "Apache-1.1")]
    Apache_1_1,
    #[serde(rename = "Apache-2.0")]
    #[default]
    Apache_2_0,
    #[serde(rename = "APAFML")]
    APAFML,
    #[serde(rename = "APL-1.0")]
    APL_1_0,
    #[serde(rename = "App-s2p")]
    App_s2p,
    #[serde(rename = "APSL-1.0")]
    APSL_1_0,
    #[serde(rename = "APSL-1.1")]
    APSL_1_1,
    #[serde(rename = "APSL-1.2")]
    APSL_1_2,
    #[serde(rename = "APSL-2.0")]
    APSL_2_0,
    #[serde(rename = "Arphic-1999")]
    Arphic_1999,
    #[serde(rename = "Artistic-1.0")]
    Artistic_1_0,
    #[serde(rename = "Artistic-1.0-cl8")]
    Artistic_1_0_cl8,
    #[serde(rename = "Artistic-1.0-Perl")]
    Artistic_1_0_Perl,
    #[serde(rename = "Artistic-2.0")]
    Artistic_2_0,
    #[serde(rename = "ASWF-Digital-Assets-1.0")]
    ASWF_Digital_Assets_1_0,
    #[serde(rename = "ASWF-Digital-Assets-1.1")]
    ASWF_Digital_Assets_1_1,
    #[serde(rename = "Baekmuk")]
    Baekmuk,
    #[serde(rename = "Bahyph")]
    Bahyph,
    #[serde(rename = "Barr")]
    Barr,
    #[serde(rename = "Beerware")]
    Beerware,
    #[serde(rename = "Bitstream-Charter")]
    Bitstream_Charter,
    #[serde(rename = "Bitstream-Vera")]
    Bitstream_Vera,
    #[serde(rename = "BitTorrent-1.0")]
    BitTorrent_1_0,
    #[serde(rename = "BitTorrent-1.1")]
    BitTorrent_1_1,
    #[serde(rename = "blessing")]
    blessing,
    #[serde(rename = "BlueOak-1.0.0")]
    BlueOak_1_0_0,
    #[serde(rename = "Boehm-GC")]
    Boehm_GC,
    #[serde(rename = "Borceux")]
    Borceux,
    #[serde(rename = "Brian-Gladman-3-Clause")]
    Brian_Gladman_3_Clause,
    #[serde(rename = "BSD-1-Clause")]
    BSD_1_Clause,
    #[serde(rename = "BSD-2-Clause")]
    BSD_2_Clause,
    #[serde(rename = "BSD-2-Clause-Patent")]
    BSD_2_Clause_Patent,
    #[serde(rename = "BSD-2-Clause-Views")]
    BSD_2_Clause_Views,
    #[serde(rename = "BSD-3-Clause")]
    BSD_3_Clause,
    #[serde(rename = "BSD-3-Clause-Attribution")]
    BSD_3_Clause_Attribution,
    #[serde(rename = "BSD-3-Clause-Clear")]
    BSD_3_Clause_Clear,
    #[serde(rename = "BSD-3-Clause-LBNL")]
    BSD_3_Clause_LBNL,
    #[serde(rename = "BSD-3-Clause-Modification")]
    BSD_3_Clause_Modification,
    #[serde(rename = "BSD-3-Clause-No-Military-License")]
    BSD_3_Clause_No_Military_License,
    #[serde(rename = "BSD-3-Clause-No-Nuclear-License")]
    BSD_3_Clause_No_Nuclear_License,
    #[serde(rename = "BSD-3-Clause-No-Nuclear-License-2014")]
    BSD_3_Clause_No_Nuclear_License_2014,
    #[serde(rename = "BSD-3-Clause-No-Nuclear-Warranty")]
    BSD_3_Clause_No_Nuclear_Warranty,
    #[serde(rename = "BSD-3-Clause-Open-MPI")]
    BSD_3_Clause_Open_MPI,
    #[serde(rename = "BSD-4-Clause")]
    BSD_4_Clause,
    #[serde(rename = "BSD-4-Clause-Shortened")]
    BSD_4_Clause_Shortened,
    #[serde(rename = "BSD-4-Clause-UC")]
    BSD_4_Clause_UC,
    #[serde(rename = "BSD-4.3RENO")]
    BSD_4_3RENO,
    #[serde(rename = "BSD-4.3TAHOE")]
    BSD_4_3TAHOE,
    #[serde(rename = "BSD-Advertising-Acknowledgement")]
    BSD_Advertising_Acknowledgement,
    #[serde(rename = "BSD-Attribution-HPND-disclaimer")]
    BSD_Attribution_HPND_disclaimer,
    #[serde(rename = "BSD-Protection")]
    BSD_Protection,
    #[serde(rename = "BSD-Source-Code")]
    BSD_Source_Code,
    #[serde(rename = "BSL-1.0")]
    BSL_1_0,
    #[serde(rename = "BUSL-1.1")]
    BUSL_1_1,
    #[serde(rename = "bzip2-1.0.6")]
    bzip2_1_0_6,
    #[serde(rename = "C-UDA-1.0")]
    C_UDA_1_0,
    #[serde(rename = "CAL-1.0")]
    CAL_1_0,
    #[serde(rename = "CAL-1.0-Combined-Work-Exception")]
    CAL_1_0_Combined_Work_Exception,
    #[serde(rename = "Caldera")]
    Caldera,
    #[serde(rename = "CATOSL-1.1")]
    CATOSL_1_1,
    #[serde(rename = "CC-BY-1.0")]
    CC_BY_1_0,
    #[serde(rename = "CC-BY-2.0")]
    CC_BY_2_0,
    #[serde(rename = "CC-BY-2.5")]
    CC_BY_2_5,
    #[serde(rename = "CC-BY-2.5-AU")]
    CC_BY_2_5_AU,
    #[serde(rename = "CC-BY-3.0")]
    CC_BY_3_0,
    #[serde(rename = "CC-BY-3.0-AT")]
    CC_BY_3_0_AT,
    #[serde(rename = "CC-BY-3.0-DE")]
    CC_BY_3_0_DE,
    #[serde(rename = "CC-BY-3.0-IGO")]
    CC_BY_3_0_IGO,
    #[serde(rename = "CC-BY-3.0-NL")]
    CC_BY_3_0_NL,
    #[serde(rename = "CC-BY-3.0-US")]
    CC_BY_3_0_US,
    #[serde(rename = "CC-BY-4.0")]
    CC_BY_4_0,
    #[serde(rename = "CC-BY-NC-1.0")]
    CC_BY_NC_1_0,
    #[serde(rename = "CC-BY-NC-2.0")]
    CC_BY_NC_2_0,
    #[serde(rename = "CC-BY-NC-2.5")]
    CC_BY_NC_2_5,
    #[serde(rename = "CC-BY-NC-3.0")]
    CC_BY_NC_3_0,
    #[serde(rename = "CC-BY-NC-3.0-DE")]
    CC_BY_NC_3_0_DE,
    #[serde(rename = "CC-BY-NC-4.0")]
    CC_BY_NC_4_0,
    #[serde(rename = "CC-BY-NC-ND-1.0")]
    CC_BY_NC_ND_1_0,
    #[serde(rename = "CC-BY-NC-ND-2.0")]
    CC_BY_NC_ND_2_0,
    #[serde(rename = "CC-BY-NC-ND-2.5")]
    CC_BY_NC_ND_2_5,
    #[serde(rename = "CC-BY-NC-ND-3.0")]
    CC_BY_NC_ND_3_0,
    #[serde(rename = "CC-BY-NC-ND-3.0-DE")]
    CC_BY_NC_ND_3_0_DE,
    #[serde(rename = "CC-BY-NC-ND-3.0-IGO")]
    CC_BY_NC_ND_3_0_IGO,
    #[serde(rename = "CC-BY-NC-ND-4.0")]
    CC_BY_NC_ND_4_0,
    #[serde(rename = "CC-BY-NC-SA-1.0")]
    CC_BY_NC_SA_1_0,
    #[serde(rename = "CC-BY-NC-SA-2.0")]
    CC_BY_NC_SA_2_0,
    #[serde(rename = "CC-BY-NC-SA-2.0-DE")]
    CC_BY_NC_SA_2_0_DE,
    #[serde(rename = "CC-BY-NC-SA-2.0-FR")]
    CC_BY_NC_SA_2_0_FR,
    #[serde(rename = "CC-BY-NC-SA-2.0-UK")]
    CC_BY_NC_SA_2_0_UK,
    #[serde(rename = "CC-BY-NC-SA-2.5")]
    CC_BY_NC_SA_2_5,
    #[serde(rename = "CC-BY-NC-SA-3.0")]
    CC_BY_NC_SA_3_0,
    #[serde(rename = "CC-BY-NC-SA-3.0-DE")]
    CC_BY_NC_SA_3_0_DE,
    #[serde(rename = "CC-BY-NC-SA-3.0-IGO")]
    CC_BY_NC_SA_3_0_IGO,
    #[serde(rename = "CC-BY-NC-SA-4.0")]
    CC_BY_NC_SA_4_0,
    #[serde(rename = "CC-BY-ND-1.0")]
    CC_BY_ND_1_0,
    #[serde(rename = "CC-BY-ND-2.0")]
    CC_BY_ND_2_0,
    #[serde(rename = "CC-BY-ND-2.5")]
    CC_BY_ND_2_5,
    #[serde(rename = "CC-BY-ND-3.0")]
    CC_BY_ND_3_0,
    #[serde(rename = "CC-BY-ND-3.0-DE")]
    CC_BY_ND_3_0_DE,
    #[serde(rename = "CC-BY-ND-4.0")]
    CC_BY_ND_4_0,
    #[serde(rename = "CC-BY-SA-1.0")]
    CC_BY_SA_1_0,
    #[serde(rename = "CC-BY-SA-2.0")]
    CC_BY_SA_2_0,
    #[serde(rename = "CC-BY-SA-2.0-UK")]
    CC_BY_SA_2_0_UK,
    #[serde(rename = "CC-BY-SA-2.1-JP")]
    CC_BY_SA_2_1_JP,
    #[serde(rename = "CC-BY-SA-2.5")]
    CC_BY_SA_2_5,
    #[serde(rename = "CC-BY-SA-3.0")]
    CC_BY_SA_3_0,
    #[serde(rename = "CC-BY-SA-3.0-AT")]
    CC_BY_SA_3_0_AT,
    #[serde(rename = "CC-BY-SA-3.0-DE")]
    CC_BY_SA_3_0_DE,
    #[serde(rename = "CC-BY-SA-3.0-IGO")]
    CC_BY_SA_3_0_IGO,
    #[serde(rename = "CC-BY-SA-4.0")]
    CC_BY_SA_4_0,
    #[serde(rename = "CC-PDDC")]
    CC_PDDC,
    #[serde(rename = "CC0-1.0")]
    CC0_1_0,
    #[serde(rename = "CDDL-1.0")]
    CDDL_1_0,
    #[serde(rename = "CDDL-1.1")]
    CDDL_1_1,
    #[serde(rename = "CDL-1.0")]
    CDL_1_0,
    #[serde(rename = "CDLA-Permissive-1.0")]
    CDLA_Permissive_1_0,
    #[serde(rename = "CDLA-Permissive-2.0")]
    CDLA_Permissive_2_0,
    #[serde(rename = "CDLA-Sharing-1.0")]
    CDLA_Sharing_1_0,
    #[serde(rename = "CECILL-1.0")]
    CECILL_1_0,
    #[serde(rename = "CECILL-1.1")]
    CECILL_1_1,
    #[serde(rename = "CECILL-2.0")]
    CECILL_2_0,
    #[serde(rename = "CECILL-2.1")]
    CECILL_2_1,
    #[serde(rename = "CECILL-B")]
    CECILL_B,
    #[serde(rename = "CECILL-C")]
    CECILL_C,
    #[serde(rename = "CERN-OHL-1.1")]
    CERN_OHL_1_1,
    #[serde(rename = "CERN-OHL-1.2")]
    CERN_OHL_1_2,
    #[serde(rename = "CERN-OHL-P-2.0")]
    CERN_OHL_P_2_0,
    #[serde(rename = "CERN-OHL-S-2.0")]
    CERN_OHL_S_2_0,
    #[serde(rename = "CERN-OHL-W-2.0")]
    CERN_OHL_W_2_0,
    #[serde(rename = "CFITSIO")]
    CFITSIO,
    #[serde(rename = "checkmk")]
    checkmk,
    #[serde(rename = "ClArtistic")]
    ClArtistic,
    #[serde(rename = "Clips")]
    Clips,
    #[serde(rename = "CMU-Mach")]
    CMU_Mach,
    #[serde(rename = "CNRI-Jython")]
    CNRI_Jython,
    #[serde(rename = "CNRI-Python")]
    CNRI_Python,
    #[serde(rename = "CNRI-Python-GPL-Compatible")]
    CNRI_Python_GPL_Compatible,
    #[serde(rename = "COIL-1.0")]
    COIL_1_0,
    #[serde(rename = "Community-Spec-1.0")]
    Community_Spec_1_0,
    #[serde(rename = "Condor-1.1")]
    Condor_1_1,
    #[serde(rename = "copyleft-next-0.3.0")]
    copyleft_next_0_3_0,
    #[serde(rename = "copyleft-next-0.3.1")]
    copyleft_next_0_3_1,
    #[serde(rename = "Cornell-Lossless-JPEG")]
    Cornell_Lossless_JPEG,
    #[serde(rename = "CPAL-1.0")]
    CPAL_1_0,
    #[serde(rename = "CPL-1.0")]
    CPL_1_0,
    #[serde(rename = "CPOL-1.02")]
    CPOL_1_02,
    #[serde(rename = "Crossword")]
    Crossword,
    #[serde(rename = "CrystalStacker")]
    CrystalStacker,
    #[serde(rename = "CUA-OPL-1.0")]
    CUA_OPL_1_0,
    #[serde(rename = "Cube")]
    Cube,
    #[serde(rename = "curl")]
    curl,
    #[serde(rename = "D-FSL-1.0")]
    D_FSL_1_0,
    #[serde(rename = "diffmark")]
    diffmark,
    #[serde(rename = "DL-DE-BY-2.0")]
    DL_DE_BY_2_0,
    #[serde(rename = "DOC")]
    DOC,
    #[serde(rename = "Dotseqn")]
    Dotseqn,
    #[serde(rename = "DRL-1.0")]
    DRL_1_0,
    #[serde(rename = "DSDP")]
    DSDP,
    #[serde(rename = "dtoa")]
    dtoa,
    #[serde(rename = "dvipdfm")]
    dvipdfm,
    #[serde(rename = "ECL-1.0")]
    ECL_1_0,
    #[serde(rename = "ECL-2.0")]
    ECL_2_0,
    #[serde(rename = "EFL-1.0")]
    EFL_1_0,
    #[serde(rename = "EFL-2.0")]
    EFL_2_0,
    #[serde(rename = "eGenix")]
    eGenix,
    #[serde(rename = "Elastic-2.0")]
    Elastic_2_0,
    #[serde(rename = "Entessa")]
    Entessa,
    #[serde(rename = "EPICS")]
    EPICS,
    #[serde(rename = "EPL-1.0")]
    EPL_1_0,
    #[serde(rename = "EPL-2.0")]
    EPL_2_0,
    #[serde(rename = "ErlPL-1.1")]
    ErlPL_1_1,
    #[serde(rename = "etalab-2.0")]
    etalab_2_0,
    #[serde(rename = "EUDatagrid")]
    EUDatagrid,
    #[serde(rename = "EUPL-1.0")]
    EUPL_1_0,
    #[serde(rename = "EUPL-1.1")]
    EUPL_1_1,
    #[serde(rename = "EUPL-1.2")]
    EUPL_1_2,
    #[serde(rename = "Eurosym")]
    Eurosym,
    #[serde(rename = "Fair")]
    Fair,
    #[serde(rename = "FDK-AAC")]
    FDK_AAC,
    #[serde(rename = "Frameworx-1.0")]
    Frameworx_1_0,
    #[serde(rename = "FreeBSD-DOC")]
    FreeBSD_DOC,
    #[serde(rename = "FreeImage")]
    FreeImage,
    #[serde(rename = "FSFAP")]
    FSFAP,
    #[serde(rename = "FSFUL")]
    FSFUL,
    #[serde(rename = "FSFULLR")]
    FSFULLR,
    #[serde(rename = "FSFULLRWD")]
    FSFULLRWD,
    #[serde(rename = "FTL")]
    FTL,
    #[serde(rename = "GD")]
    GD,
    #[serde(rename = "GFDL-1.1-invariants-only")]
    GFDL_1_1_invariants_only,
    #[serde(rename = "GFDL-1.1-invariants-or-later")]
    GFDL_1_1_invariants_or_later,
    #[serde(rename = "GFDL-1.1-no-invariants-only")]
    GFDL_1_1_no_invariants_only,
    #[serde(rename = "GFDL-1.1-no-invariants-or-later")]
    GFDL_1_1_no_invariants_or_later,
    #[serde(rename = "GFDL-1.1-only")]
    GFDL_1_1_only,
    #[serde(rename = "GFDL-1.1-or-later")]
    GFDL_1_1_or_later,
    #[serde(rename = "GFDL-1.2-invariants-only")]
    GFDL_1_2_invariants_only,
    #[serde(rename = "GFDL-1.2-invariants-or-later")]
    GFDL_1_2_invariants_or_later,
    #[serde(rename = "GFDL-1.2-no-invariants-only")]
    GFDL_1_2_no_invariants_only,
    #[serde(rename = "GFDL-1.2-no-invariants-or-later")]
    GFDL_1_2_no_invariants_or_later,
    #[serde(rename = "GFDL-1.2-only")]
    GFDL_1_2_only,
    #[serde(rename = "GFDL-1.2-or-later")]
    GFDL_1_2_or_later,
    #[serde(rename = "GFDL-1.3-invariants-only")]
    GFDL_1_3_invariants_only,
    #[serde(rename = "GFDL-1.3-invariants-or-later")]
    GFDL_1_3_invariants_or_later,
    #[serde(rename = "GFDL-1.3-no-invariants-only")]
    GFDL_1_3_no_invariants_only,
    #[serde(rename = "GFDL-1.3-no-invariants-or-later")]
    GFDL_1_3_no_invariants_or_later,
    #[serde(rename = "GFDL-1.3-only")]
    GFDL_1_3_only,
    #[serde(rename = "GFDL-1.3-or-later")]
    GFDL_1_3_or_later,
    #[serde(rename = "Giftware")]
    Giftware,
    #[serde(rename = "GL2PS")]
    GL2PS,
    #[serde(rename = "Glide")]
    Glide,
    #[serde(rename = "Glulxe")]
    Glulxe,
    #[serde(rename = "GLWTPL")]
    GLWTPL,
    #[serde(rename = "gnuplot")]
    gnuplot,
    #[serde(rename = "GPL-1.0-only")]
    GPL_1_0_only,
    #[serde(rename = "GPL-1.0-or-later")]
    GPL_1_0_or_later,
    #[serde(rename = "GPL-2.0-only")]
    GPL_2_0_only,
    #[serde(rename = "GPL-2.0-or-later")]
    GPL_2_0_or_later,
    #[serde(rename = "GPL-3.0-only")]
    GPL_3_0_only,
    #[serde(rename = "GPL-3.0-or-later")]
    GPL_3_0_or_later,
    #[serde(rename = "Graphics-Gems")]
    Graphics_Gems,
    #[serde(rename = "gSOAP-1.3b")]
    gSOAP_1_3b,
    #[serde(rename = "HaskellReport")]
    HaskellReport,
    #[serde(rename = "Hippocratic-2.1")]
    Hippocratic_2_1,
    #[serde(rename = "HP-1986")]
    HP_1986,
    #[serde(rename = "HPND")]
    HPND,
    #[serde(rename = "HPND-export-US")]
    HPND_export_US,
    #[serde(rename = "HPND-Markus-Kuhn")]
    HPND_Markus_Kuhn,
    #[serde(rename = "HPND-sell-variant")]
    HPND_sell_variant,
    #[serde(rename = "HPND-sell-variant-MIT-disclaimer")]
    HPND_sell_variant_MIT_disclaimer,
    #[serde(rename = "HTMLTIDY")]
    HTMLTIDY,
    #[serde(rename = "IBM-pibs")]
    IBM_pibs,
    #[serde(rename = "ICU")]
    ICU,
    #[serde(rename = "IEC-Code-Components-EULA")]
    IEC_Code_Components_EULA,
    #[serde(rename = "IJG")]
    IJG,
    #[serde(rename = "IJG-short")]
    IJG_short,
    #[serde(rename = "ImageMagick")]
    ImageMagick,
    #[serde(rename = "iMatix")]
    iMatix,
    #[serde(rename = "Imlib2")]
    Imlib2,
    #[serde(rename = "Info-ZIP")]
    Info_ZIP,
    #[serde(rename = "Inner-Net-2.0")]
    Inner_Net_2_0,
    #[serde(rename = "Intel")]
    Intel,
    #[serde(rename = "Intel-ACPI")]
    Intel_ACPI,
    #[serde(rename = "Interbase-1.0")]
    Interbase_1_0,
    #[serde(rename = "IPA")]
    IPA,
    #[serde(rename = "IPL-1.0")]
    IPL_1_0,
    #[serde(rename = "ISC")]
    ISC,
    #[serde(rename = "Jam")]
    Jam,
    #[serde(rename = "JasPer-2.0")]
    JasPer_2_0,
    #[serde(rename = "JPL-image")]
    JPL_image,
    #[serde(rename = "JPNIC")]
    JPNIC,
    #[serde(rename = "JSON")]
    JSON,
    #[serde(rename = "Kazlib")]
    Kazlib,
    #[serde(rename = "Knuth-CTAN")]
    Knuth_CTAN,
    #[serde(rename = "LAL-1.2")]
    LAL_1_2,
    #[serde(rename = "LAL-1.3")]
    LAL_1_3,
    #[serde(rename = "Latex2e")]
    Latex2e,
    #[serde(rename = "Latex2e-translated-notice")]
    Latex2e_translated_notice,
    #[serde(rename = "Leptonica")]
    Leptonica,
    #[serde(rename = "LGPL-2.0-only")]
    LGPL_2_0_only,
    #[serde(rename = "LGPL-2.0-or-later")]
    LGPL_2_0_or_later,
    #[serde(rename = "LGPL-2.1-only")]
    LGPL_2_1_only,
    #[serde(rename = "LGPL-2.1-or-later")]
    LGPL_2_1_or_later,
    #[serde(rename = "LGPL-3.0-only")]
    LGPL_3_0_only,
    #[serde(rename = "LGPL-3.0-or-later")]
    LGPL_3_0_or_later,
    #[serde(rename = "LGPLLR")]
    LGPLLR,
    #[serde(rename = "Libpng")]
    Libpng,
    #[serde(rename = "libpng-2.0")]
    libpng_2_0,
    #[serde(rename = "libselinux-1.0")]
    libselinux_1_0,
    #[serde(rename = "libtiff")]
    libtiff,
    #[serde(rename = "libutil-David-Nugent")]
    libutil_David_Nugent,
    #[serde(rename = "LiLiQ-P-1.1")]
    LiLiQ_P_1_1,
    #[serde(rename = "LiLiQ-R-1.1")]
    LiLiQ_R_1_1,
    #[serde(rename = "LiLiQ-Rplus-1.1")]
    LiLiQ_Rplus_1_1,
    #[serde(rename = "Linux-man-pages-1-para")]
    Linux_man_pages_1_para,
    #[serde(rename = "Linux-man-pages-copyleft")]
    Linux_man_pages_copyleft,
    #[serde(rename = "Linux-man-pages-copyleft-2-para")]
    Linux_man_pages_copyleft_2_para,
    #[serde(rename = "Linux-man-pages-copyleft-var")]
    Linux_man_pages_copyleft_var,
    #[serde(rename = "Linux-OpenIB")]
    Linux_OpenIB,
    #[serde(rename = "LOOP")]
    LOOP,
    #[serde(rename = "LPL-1.0")]
    LPL_1_0,
    #[serde(rename = "LPL-1.02")]
    LPL_1_02,
    #[serde(rename = "LPPL-1.0")]
    LPPL_1_0,
    #[serde(rename = "LPPL-1.1")]
    LPPL_1_1,
    #[serde(rename = "LPPL-1.2")]
    LPPL_1_2,
    #[serde(rename = "LPPL-1.3a")]
    LPPL_1_3a,
    #[serde(rename = "LPPL-1.3c")]
    LPPL_1_3c,
    #[serde(rename = "LZMA-SDK-9.11-to-9.20")]
    LZMA_SDK_9_11_to_9_20,
    #[serde(rename = "LZMA-SDK-9.22")]
    LZMA_SDK_9_22,
    #[serde(rename = "MakeIndex")]
    MakeIndex,
    #[serde(rename = "Martin-Birgmeier")]
    Martin_Birgmeier,
    #[serde(rename = "metamail")]
    metamail,
    #[serde(rename = "Minpack")]
    Minpack,
    #[serde(rename = "MirOS")]
    MirOS,
    #[serde(rename = "MIT")]
    MIT,
    #[serde(rename = "MIT-0")]
    MIT_0,
    #[serde(rename = "MIT-advertising")]
    MIT_advertising,
    #[serde(rename = "MIT-CMU")]
    MIT_CMU,
    #[serde(rename = "MIT-enna")]
    MIT_enna,
    #[serde(rename = "MIT-feh")]
    MIT_feh,
    #[serde(rename = "MIT-Festival")]
    MIT_Festival,
    #[serde(rename = "MIT-Modern-Variant")]
    MIT_Modern_Variant,
    #[serde(rename = "MIT-open-group")]
    MIT_open_group,
    #[serde(rename = "MIT-Wu")]
    MIT_Wu,
    #[serde(rename = "MITNFA")]
    MITNFA,
    #[serde(rename = "Motosoto")]
    Motosoto,
    #[serde(rename = "mpi-permissive")]
    mpi_permissive,
    #[serde(rename = "mpich2")]
    mpich2,
    #[serde(rename = "MPL-1.0")]
    MPL_1_0,
    #[serde(rename = "MPL-1.1")]
    MPL_1_1,
    #[serde(rename = "MPL-2.0")]
    MPL_2_0,
    #[serde(rename = "MPL-2.0-no-copyleft-exception")]
    MPL_2_0_no_copyleft_exception,
    #[serde(rename = "mplus")]
    mplus,
    #[serde(rename = "MS-LPL")]
    MS_LPL,
    #[serde(rename = "MS-PL")]
    MS_PL,
    #[serde(rename = "MS-RL")]
    MS_RL,
    #[serde(rename = "MTLL")]
    MTLL,
    #[serde(rename = "MulanPSL-1.0")]
    MulanPSL_1_0,
    #[serde(rename = "MulanPSL-2.0")]
    MulanPSL_2_0,
    #[serde(rename = "Multics")]
    Multics,
    #[serde(rename = "Mup")]
    Mup,
    #[serde(rename = "NAIST-2003")]
    NAIST_2003,
    #[serde(rename = "NASA-1.3")]
    NASA_1_3,
    #[serde(rename = "Naumen")]
    Naumen,
    #[serde(rename = "NBPL-1.0")]
    NBPL_1_0,
    #[serde(rename = "NCGL-UK-2.0")]
    NCGL_UK_2_0,
    #[serde(rename = "NCSA")]
    NCSA,
    #[serde(rename = "Net-SNMP")]
    Net_SNMP,
    #[serde(rename = "NetCDF")]
    NetCDF,
    #[serde(rename = "Newsletr")]
    Newsletr,
    #[serde(rename = "NGPL")]
    NGPL,
    #[serde(rename = "NICTA-1.0")]
    NICTA_1_0,
    #[serde(rename = "NIST-PD")]
    NIST_PD,
    #[serde(rename = "NIST-PD-fallback")]
    NIST_PD_fallback,
    #[serde(rename = "NIST-Software")]
    NIST_Software,
    #[serde(rename = "NLOD-1.0")]
    NLOD_1_0,
    #[serde(rename = "NLOD-2.0")]
    NLOD_2_0,
    #[serde(rename = "NLPL")]
    NLPL,
    #[serde(rename = "Nokia")]
    Nokia,
    #[serde(rename = "NOSL")]
    NOSL,
    #[serde(rename = "Noweb")]
    Noweb,
    #[serde(rename = "NPL-1.0")]
    NPL_1_0,
    #[serde(rename = "NPL-1.1")]
    NPL_1_1,
    #[serde(rename = "NPOSL-3.0")]
    NPOSL_3_0,
    #[serde(rename = "NRL")]
    NRL,
    #[serde(rename = "NTP")]
    NTP,
    #[serde(rename = "NTP-0")]
    NTP_0,
    #[serde(rename = "O-UDA-1.0")]
    O_UDA_1_0,
    #[serde(rename = "OCCT-PL")]
    OCCT_PL,
    #[serde(rename = "OCLC-2.0")]
    OCLC_2_0,
    #[serde(rename = "ODbL-1.0")]
    ODbL_1_0,
    #[serde(rename = "ODC-By-1.0")]
    ODC_By_1_0,
    #[serde(rename = "OFFIS")]
    OFFIS,
    #[serde(rename = "OFL-1.0")]
    OFL_1_0,
    #[serde(rename = "OFL-1.0-no-RFN")]
    OFL_1_0_no_RFN,
    #[serde(rename = "OFL-1.0-RFN")]
    OFL_1_0_RFN,
    #[serde(rename = "OFL-1.1")]
    OFL_1_1,
    #[serde(rename = "OFL-1.1-no-RFN")]
    OFL_1_1_no_RFN,
    #[serde(rename = "OFL-1.1-RFN")]
    OFL_1_1_RFN,
    #[serde(rename = "OGC-1.0")]
    OGC_1_0,
    #[serde(rename = "OGDL-Taiwan-1.0")]
    OGDL_Taiwan_1_0,
    #[serde(rename = "OGL-Canada-2.0")]
    OGL_Canada_2_0,
    #[serde(rename = "OGL-UK-1.0")]
    OGL_UK_1_0,
    #[serde(rename = "OGL-UK-2.0")]
    OGL_UK_2_0,
    #[serde(rename = "OGL-UK-3.0")]
    OGL_UK_3_0,
    #[serde(rename = "OGTSL")]
    OGTSL,
    #[serde(rename = "OLDAP-1.1")]
    OLDAP_1_1,
    #[serde(rename = "OLDAP-1.2")]
    OLDAP_1_2,
    #[serde(rename = "OLDAP-1.3")]
    OLDAP_1_3,
    #[serde(rename = "OLDAP-1.4")]
    OLDAP_1_4,
    #[serde(rename = "OLDAP-2.0")]
    OLDAP_2_0,
    #[serde(rename = "OLDAP-2.0.1")]
    OLDAP_2_0_1,
    #[serde(rename = "OLDAP-2.1")]
    OLDAP_2_1,
    #[serde(rename = "OLDAP-2.2")]
    OLDAP_2_2,
    #[serde(rename = "OLDAP-2.2.1")]
    OLDAP_2_2_1,
    #[serde(rename = "OLDAP-2.2.2")]
    OLDAP_2_2_2,
    #[serde(rename = "OLDAP-2.3")]
    OLDAP_2_3,
    #[serde(rename = "OLDAP-2.4")]
    OLDAP_2_4,
    #[serde(rename = "OLDAP-2.5")]
    OLDAP_2_5,
    #[serde(rename = "OLDAP-2.6")]
    OLDAP_2_6,
    #[serde(rename = "OLDAP-2.7")]
    OLDAP_2_7,
    #[serde(rename = "OLDAP-2.8")]
    OLDAP_2_8,
    #[serde(rename = "OLFL-1.3")]
    OLFL_1_3,
    #[serde(rename = "OML")]
    OML,
    #[serde(rename = "OpenPBS-2.3")]
    OpenPBS_2_3,
    #[serde(rename = "OpenSSL")]
    OpenSSL,
    #[serde(rename = "OPL-1.0")]
    OPL_1_0,
    #[serde(rename = "OPL-UK-3.0")]
    OPL_UK_3_0,
    #[serde(rename = "OPUBL-1.0")]
    OPUBL_1_0,
    #[serde(rename = "OSET-PL-2.1")]
    OSET_PL_2_1,
    #[serde(rename = "OSL-1.0")]
    OSL_1_0,
    #[serde(rename = "OSL-1.1")]
    OSL_1_1,
    #[serde(rename = "OSL-2.0")]
    OSL_2_0,
    #[serde(rename = "OSL-2.1")]
    OSL_2_1,
    #[serde(rename = "OSL-3.0")]
    OSL_3_0,
    #[serde(rename = "Parity-6.0.0")]
    Parity_6_0_0,
    #[serde(rename = "Parity-7.0.0")]
    Parity_7_0_0,
    #[serde(rename = "PDDL-1.0")]
    PDDL_1_0,
    #[serde(rename = "PHP-3.0")]
    PHP_3_0,
    #[serde(rename = "PHP-3.01")]
    PHP_3_01,
    #[serde(rename = "Plexus")]
    Plexus,
    #[serde(rename = "PolyForm-Noncommercial-1.0.0")]
    PolyForm_Noncommercial_1_0_0,
    #[serde(rename = "PolyForm-Small-Business-1.0.0")]
    PolyForm_Small_Business_1_0_0,
    #[serde(rename = "PostgreSQL")]
    PostgreSQL,
    #[serde(rename = "PSF-2.0")]
    PSF_2_0,
    #[serde(rename = "psfrag")]
    psfrag,
    #[serde(rename = "psutils")]
    psutils,
    #[serde(rename = "Python-2.0")]
    Python_2_0,
    #[serde(rename = "Python-2.0.1")]
    Python_2_0_1,
    #[serde(rename = "Qhull")]
    Qhull,
    #[serde(rename = "QPL-1.0")]
    QPL_1_0,
    #[serde(rename = "QPL-1.0-INRIA-2004")]
    QPL_1_0_INRIA_2004,
    #[serde(rename = "Rdisc")]
    Rdisc,
    #[serde(rename = "RHeCos-1.1")]
    RHeCos_1_1,
    #[serde(rename = "RPL-1.1")]
    RPL_1_1,
    #[serde(rename = "RPL-1.5")]
    RPL_1_5,
    #[serde(rename = "RPSL-1.0")]
    RPSL_1_0,
    #[serde(rename = "RSA-MD")]
    RSA_MD,
    #[serde(rename = "RSCPL")]
    RSCPL,
    #[serde(rename = "Ruby")]
    Ruby,
    #[serde(rename = "SAX-PD")]
    SAX_PD,
    #[serde(rename = "Saxpath")]
    Saxpath,
    #[serde(rename = "SCEA")]
    SCEA,
    #[serde(rename = "SchemeReport")]
    SchemeReport,
    #[serde(rename = "Sendmail")]
    Sendmail,
    #[serde(rename = "Sendmail-8.23")]
    Sendmail_8_23,
    #[serde(rename = "SGI-B-1.0")]
    SGI_B_1_0,
    #[serde(rename = "SGI-B-1.1")]
    SGI_B_1_1,
    #[serde(rename = "SGI-B-2.0")]
    SGI_B_2_0,
    #[serde(rename = "SGP4")]
    SGP4,
    #[serde(rename = "SHL-0.5")]
    SHL_0_5,
    #[serde(rename = "SHL-0.51")]
    SHL_0_51,
    #[serde(rename = "SimPL-2.0")]
    SimPL_2_0,
    #[serde(rename = "SISSL")]
    SISSL,
    #[serde(rename = "SISSL-1.2")]
    SISSL_1_2,
    #[serde(rename = "Sleepycat")]
    Sleepycat,
    #[serde(rename = "SMLNJ")]
    SMLNJ,
    #[serde(rename = "SMPPL")]
    SMPPL,
    #[serde(rename = "SNIA")]
    SNIA,
    #[serde(rename = "snprintf")]
    snprintf,
    #[serde(rename = "Spencer-86")]
    Spencer_86,
    #[serde(rename = "Spencer-94")]
    Spencer_94,
    #[serde(rename = "Spencer-99")]
    Spencer_99,
    #[serde(rename = "SPL-1.0")]
    SPL_1_0,
    #[serde(rename = "SSH-OpenSSH")]
    SSH_OpenSSH,
    #[serde(rename = "SSH-short")]
    SSH_short,
    #[serde(rename = "SSPL-1.0")]
    SSPL_1_0,
    #[serde(rename = "SugarCRM-1.1.3")]
    SugarCRM_1_1_3,
    #[serde(rename = "SunPro")]
    SunPro,
    #[serde(rename = "SWL")]
    SWL,
    #[serde(rename = "Symlinks")]
    Symlinks,
    #[serde(rename = "TAPR-OHL-1.0")]
    TAPR_OHL_1_0,
    #[serde(rename = "TCL")]
    TCL,
    #[serde(rename = "TCP-wrappers")]
    TCP_wrappers,
    #[serde(rename = "TermReadKey")]
    TermReadKey,
    #[serde(rename = "TMate")]
    TMate,
    #[serde(rename = "TORQUE-1.1")]
    TORQUE_1_1,
    #[serde(rename = "TOSL")]
    TOSL,
    #[serde(rename = "TPDL")]
    TPDL,
    #[serde(rename = "TPL-1.0")]
    TPL_1_0,
    #[serde(rename = "TTWL")]
    TTWL,
    #[serde(rename = "TU-Berlin-1.0")]
    TU_Berlin_1_0,
    #[serde(rename = "TU-Berlin-2.0")]
    TU_Berlin_2_0,
    #[serde(rename = "UCAR")]
    UCAR,
    #[serde(rename = "UCL-1.0")]
    UCL_1_0,
    #[serde(rename = "Unicode-DFS-2015")]
    Unicode_DFS_2015,
    #[serde(rename = "Unicode-DFS-2016")]
    Unicode_DFS_2016,
    #[serde(rename = "Unicode-TOU")]
    Unicode_TOU,
    #[serde(rename = "UnixCrypt")]
    UnixCrypt,
    #[serde(rename = "Unlicense")]
    Unlicense,
    #[serde(rename = "UPL-1.0")]
    UPL_1_0,
    #[serde(rename = "Vim")]
    Vim,
    #[serde(rename = "VOSTROM")]
    VOSTROM,
    #[serde(rename = "VSL-1.0")]
    VSL_1_0,
    #[serde(rename = "W3C")]
    W3C,
    #[serde(rename = "W3C-19980720")]
    W3C_19980720,
    #[serde(rename = "W3C-20150513")]
    W3C_20150513,
    #[serde(rename = "w3m")]
    w3m,
    #[serde(rename = "Watcom-1.0")]
    Watcom_1_0,
    #[serde(rename = "Widget-Workshop")]
    Widget_Workshop,
    #[serde(rename = "Wsuipa")]
    Wsuipa,
    #[serde(rename = "WTFPL")]
    WTFPL,
    #[serde(rename = "X11")]
    X11,
    #[serde(rename = "X11-distribute-modifications-variant")]
    X11_distribute_modifications_variant,
    #[serde(rename = "Xdebug-1.03")]
    Xdebug_1_03,
    #[serde(rename = "Xerox")]
    Xerox,
    #[serde(rename = "Xfig")]
    Xfig,
    #[serde(rename = "XFree86-1.1")]
    XFree86_1_1,
    #[serde(rename = "xinetd")]
    xinetd,
    #[serde(rename = "xlock")]
    xlock,
    #[serde(rename = "Xnet")]
    Xnet,
    #[serde(rename = "xpp")]
    xpp,
    #[serde(rename = "XSkat")]
    XSkat,
    #[serde(rename = "YPL-1.0")]
    YPL_1_0,
    #[serde(rename = "YPL-1.1")]
    YPL_1_1,
    #[serde(rename = "Zed")]
    Zed,
    #[serde(rename = "Zend-2.0")]
    Zend_2_0,
    #[serde(rename = "Zimbra-1.3")]
    Zimbra_1_3,
    #[serde(rename = "Zimbra-1.4")]
    Zimbra_1_4,
    #[serde(rename = "Zlib")]
    Zlib,
    #[serde(rename = "zlib-acknowledgement")]
    zlib_acknowledgement,
    #[serde(rename = "ZPL-1.1")]
    ZPL_1_1,
    #[serde(rename = "ZPL-2.0")]
    ZPL_2_0,
    #[serde(rename = "ZPL-2.1")]
    ZPL_2_1,
}

impl AsPartial for LicenseId {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        self.to_string()
    }
}


#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum DeprecatedLicenseId{
    #[serde(rename="AGPL-1.0")]
    AGPL_1_0,
    #[serde(rename="AGPL-3.0")]
    AGPL_3_0,
    #[serde(rename="BSD-2-Clause-FreeBSD")]
    BSD_2_Clause_FreeBSD,
    #[serde(rename="BSD-2-Clause-NetBSD")]
    BSD_2_Clause_NetBSD,
    #[serde(rename="bzip2-1.0.5")]
    bzip2_1_0_5,
    #[serde(rename="eCos-2.0")]
    eCos_2_0,
    #[serde(rename="GFDL-1.1")]
    GFDL_1_1,
    #[serde(rename="GFDL-1.2")]
    GFDL_1_2,
    #[serde(rename="GFDL-1.3")]
    GFDL_1_3,
    #[serde(rename="GPL-1.0")]
    GPL_1_0,
    #[serde(rename="GPL-1.0+")]
    GPL_1_0_plus,
    #[serde(rename="GPL-2.0")]
    GPL_2_0,
    #[serde(rename="GPL-2.0+")]
    GPL_2_0_plus,
    #[serde(rename="GPL-2.0-with-autoconf-exception")]
    GPL_2_0_with_autoconf_exception,
    #[serde(rename="GPL-2.0-with-bison-exception")]
    GPL_2_0_with_bison_exception,
    #[serde(rename="GPL-2.0-with-classpath-exception")]
    GPL_2_0_with_classpath_exception,
    #[serde(rename="GPL-2.0-with-font-exception")]
    GPL_2_0_with_font_exception,
    #[serde(rename="GPL-2.0-with-GCC-exception")]
    GPL_2_0_with_GCC_exception,
    #[serde(rename="GPL-3.0")]
    GPL_3_0,
    #[serde(rename="GPL-3.0+")]
    GPL_3_0_plus,
    #[serde(rename="GPL-3.0-with-autoconf-exception")]
    GPL_3_0_with_autoconf_exception,
    #[serde(rename="GPL-3.0-with-GCC-exception")]
    GPL_3_0_with_GCC_exception,
    #[serde(rename="LGPL-2.0")]
    LGPL_2_0,
    #[serde(rename="LGPL-2.0+")]
    LGPL_2_0_plus,
    #[serde(rename="LGPL-2.1")]
    LGPL_2_1,
    #[serde(rename="LGPL-2.1+")]
    LGPL_2_1_plus,
    #[serde(rename="LGPL-3.0")]
    LGPL_3_0,
    #[serde(rename="LGPL-3.0+")]
    LGPL_3_0_plus,
    #[serde(rename="Nunit")]
    Nunit,
    #[serde(rename="StandardML-NJ")]
    StandardML_NJ,
    #[serde(rename="wxWindows")]
    wxWindows,
}
