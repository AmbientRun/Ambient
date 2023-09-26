if [ -z "${EXAMPLES_ASSETS_HOST}" ]; then
    echo "EXAMPLES_ASSETS_HOST is not set"
    exit 1
fi

export BASE="${EXAMPLES_ASSETS_HOST}/unity/Dynamic%20Nature%20-%20Mountain%20Tree%20Pack"
for value in \
    /Prefabs/Standard/Fir_01_Plant.prefab \
    /Models/Fir_01_Plant.FBX \
    /Models/Fir_01_Plant.FBX.meta \
    /Models/Fir_01_Plant_cross.asset \
    /Models/Fir_01_Plant_cross.asset.meta \
    /Models/Fir_01_Plant_cross_s.asset \
    /Models/Fir_01_Plant_cross_s.asset.meta \
    /Models/Materials/M_Fir_01_Cross.mat \
    /Models/Materials/M_Fir_01_Cross.mat.meta \
    /Models/Materials/M_Fir_Bark_01.mat \
    /Models/Materials/M_Fir_Bark_01.mat.meta \
    /Models/Materials/M_Fir_Bark_01_06.mat \
    /Models/Materials/M_Fir_Bark_01_06.mat.meta \
    /Models/Materials/M_Fir_Leaves.mat \
    /Models/Materials/M_Fir_Leaves.mat.meta \
    /Models/Textures/T_Fir_01_Atlas.png \
    /Models/Textures/T_Fir_01_Atlas.png.meta \
    /Models/Textures/T_Fir_01_Atlas_T.png \
    /Models/Textures/T_Fir_01_Atlas_T.png.meta \
    /Models/Textures/T_Fir_01_Atlas_n.png \
    /Models/Textures/T_Fir_01_Atlas_n.png.meta \
    /Models/Textures/T_Fir_bark_01_BC.tga \
    /Models/Textures/T_Fir_bark_01_BC.tga.meta \
    /Models/Textures/T_Fir_bark_01_MT_AO_G.tga \
    /Models/Textures/T_Fir_bark_01_MT_AO_G.tga.meta \
    /Models/Textures/T_Fir_leaves_BC_T.TGA \
    /Models/Textures/T_Fir_leaves_BC_T.TGA.meta \
    /Models/Textures/T_Fir_leaves_MT_AO_G.tga \
    /Models/Textures/T_Fir_leaves_MT_AO_G.tga.meta \
    /Models/Textures/T_Fir_leaves_N.TGA \
    /Models/Textures/T_Fir_leaves_N.TGA.meta 
do
   echo Downloading ${value}
   curl ${BASE}/${value} --create-dirs -o assets/TreePack/${value}
done
