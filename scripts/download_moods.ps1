New-Item -ItemType Directory -Force assets\moods | Out-Null
$items = @{
    'chill' = 'rAuzuLfViDJUshG4WU2sHbQv6hFX6ZCnGpI-_ucalfi9RGDoZisfPx9iESN1--HSyeCmTc2YKfJO4wM';
    'commute' = 'SMpT_3hW0yjMrAoiZ3EhiGP45ettuyjdSxJ2dHShO1N8hg1EOkBtv0sBRApF4bsSmIZw-w4BGilxMc8';
    'diwali' = 'Ey-p72hOgNHfU0NxW6Ai1CD67kaxKeOll7agJg7G25BMcPlRNrkVpUn1rmFK_psiNdIAjueXype4o2WG';
    'energize' = 'eHXeI8ZwBw_VIoPjnmKuCs2TBuWMfLe2x3zrLQcnGsC1tfO5uzHBFD2dCi7ZhpcUlW8tS0NbE04hgUsU';
    'feel_good' = '5F28fEHuZr_xIDxhhrku9zYos278p9jDrxGs_OGHqRMVxmqmRRnqtOZkl9cuymSCOwx_ymD7pky-BQ';
    'focus' = 'Ql0IVQFj5EjQdzn4FJpbjnXlPMlmSWYtudKxhl_E_ZeCNDLPX7gHzBTk0tQQlDFGoxjrcVN53uWWf_8';
    'gaming' = 'falgLCbx514wdlBjaNsulnyLw0pPPP3tqoceRUKoI4RLcezgilNjc1L294fNRO0R1TvldgGWv20ggA';
    'party' = 'WhhEXIUv8zXrDDLaZmT1tOMmlls8JPOvu_mzwC5a1QNBEwOrBI-sFambyKy5lF4SpInsRDXbrYLB9Q';
    'romance' = 'NTW2SQcwpsRpnY5S6pyymw82zlXebkeTsfly0h_RHEZG6DO1FPWT7buULrkPNtAtwxJMkyLWIRw-p_4';
    'sad' = '0K0J7aA-vQaLqc71ZqSoDsZR5x0dIvjzJYLQfCoYSiCH18dvLXnwFJ9UYmQ1nLyUSXGPglupcHBhBqid';
    'sleep' = 'uABch78iXzNqPW5ot6KpF6cqrVqhx1c3L7VGRGqZDoV-0ZkjkodzE2nZH5sk7hRK59dHNWvdv7eSkhbW';
    'workout' = 'vccnCaZw3ie3ITUhRsRxOUwikeMmlL_xWIA5ilyx3A1ydiLt54tCc2hCJ_h0EkHgwSbImiX_XhMC-g';
    'african' = '5iEV4BXK_LQvlOnb4fiS_6mn9ux0nmS3MSkYoG7kePZjPXNxDQOGFTZBU707BIK78rk4TyPWt2k1swo';
    'arabic' = 'Ox_xJNYqzLkHYdPcmtdCAhnsmAXkgKTNuhM2HGcfaXEq3hnkQAVK7rhV7ZerPCcIQ4tpVIzBIL2YFmo';
    'bengali' = 'twtZoAVWxJUgUk62bPEPDuxqXPEkReKeIc_31xi2jNRagPLYzBG356LUbBMB_xHJfbWWphbLWaIcsRA';
    'decades' = 'NPNY9n7Klaje5alEv_toJoRrjG-vaD2T9hdsiZKIA23kDU0jKQHZBI66-geguNEGtVp-ZwXTYdV7gLw';
    'devotional' = 'WlWKWCVEhM2Tn6oZuY69QLCtU6J6rC0r5gWLuRLXeMNqWha9yYJXV9frC3-SSVvBvHr9TdtYIb7HJQ';
    'family' = 'R3LjYWeO4nKV-3BvqP4ESUFndRuyEVD68DLOKX2KpY-VW-Ran80TCeo10TEjUESONK0MbbhgrroE7v1J';
    'folk' = 'J0m3mhwQYktptEg8IouqUm8EuO3u_sm0drLpNQCdA1PHmLkdfVsZY64Qi15_uqyIAbh1Fp6ixgLZAOz8';
    'indian_indie' = 'Kz_AUVlrDSAEQ405NwMs94RMNe_65iiEpuw1P5xJe1-cuOirWXMw_LLLeCRUlKH5xaFbSfwVp7r6-A';
    'jpop' = 'Vry7iLCt6cTHbTEEKH4t3-_WgRqkSFvqHjRn0j3vY2VbFMUPfpzvoEWBK3XdlBB_aBm3OmKOdzUi-8A';
    'jazz' = 'vtO-yvOzP9gdAIxfReYHTP-cICOzdEidVbt34vYpAmLGwZJTJJTzrn64YYoO2Ll33MLP--e1ZSwfEOV2';
    'kpop' = 'Bse0tDvaNbxhbcxhprmh-0KEs04SgHG4rDwqqIeaWVSL9bDcvTKi6RVMJvUUDSzSBQ_5DZN76NwkX3I';
    'latin' = 'gxO4oCQCYLwC2jMivZ9nLqEvWOoc73U8hPYt4ejSOk5gXmBgevuyYFCoODBOa754Yt65BeRE0oiqK5Q';
    'metal' = '9Q3Bg10TTEpckT8loX0eSPzkxAGiD8mIAw6MgKHI92MNHvSQk2KtbGjwIrNOMKpp5Qw1QHoen8WvO5ct'
}
foreach ($name in $items.Keys) {
    $url = "https://lh3.googleusercontent.com/$($items[$name])=w80-h80-l75-rj"
    $file = "assets\moods\$name.jpg"
    Invoke-WebRequest -Uri $url -OutFile $file
}
$count = (Get-ChildItem assets\moods).Count
Write-Host "Downloaded $count images into assets\moods"

