struct VSOut {
    float3 color : Color;
    float4 pos : SV_Position;
};

cbuffer CBuf{
    matrix transform;
};

VSOut main(float3 pos : Position, float3 color : Color) {
    VSOut vsOut;
    vsOut.pos = mul(float4(pos, 1.0f), transform);
    vsOut.color = color;
    return vsOut;
};
